use alloy_primitives::{B256, Bytes, FixedBytes};
use quick_impl::quick_impl;

use crate::{StorageNode, StorageNodeChildren, utils::slice_is_zero};

#[auto_impl::auto_impl(&mut)]
pub trait StorageReader {
    fn next<B: SubB256>(&mut self) -> Result<StorageReaderNext<B>, RemainingBytes>;

    fn next_or_default<B: SubB256>(&mut self) -> StorageReaderNext<B>;

    fn consume_remaining(&mut self) -> RemainingBytes;
}

pub trait IntoStorageReader {
    fn into_storage_reader(self) -> impl StorageReader;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[quick_impl]
pub struct RemainingBytes(#[quick_impl(impl Deref, impl DerefMut, impl From, impl Into)] pub Bytes);

impl RemainingBytes {
    pub fn is_zero(&self) -> bool {
        slice_is_zero(&self.0)
    }

    pub fn is_not_zero(&self) -> bool {
        !self.is_zero()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct StorageReaderNext<B> {
    pub word: B,
    pub children: StorageNodeChildren,
    pub remaining: RemainingBytes,
}

impl<B> StorageReaderNext<B> {
    pub fn is_remaining_zero(&self) -> bool {
        self.remaining.is_zero()
    }

    pub fn is_remaining_not_zero(&self) -> bool {
        self.remaining.is_not_zero()
    }
}

#[derive(Debug, Clone)]
pub struct StorageReaderImpl<I> {
    iterator: I,
    current: Option<(B256Reader<true>, StorageNodeChildren)>,
}

impl<I> StorageReaderImpl<I> {
    pub const fn new(iterator: I) -> Self {
        Self {
            iterator,
            current: None,
        }
    }
}

impl<I> StorageReaderImpl<I>
where
    I: Iterator<Item = StorageNode>,
{
    fn next_with_remaining<B: SubB256>(
        &mut self,
        remaining: RemainingBytes,
    ) -> Result<StorageReaderNext<B>, RemainingBytes> {
        let Some((ref mut reader, ref children)) = self.current else {
            if let Some(next) = self.iterator.next() {
                let value = next.value();
                let reader = B256Reader::new(value);
                self.current = Some((reader, next.children));
            } else {
                let reader = B256Reader::new(B256::ZERO);
                self.current = Some((reader, Default::default()));
            };
            return self.next_with_remaining(remaining);
        };

        match reader.next::<B>() {
            Ok(word) => Ok(StorageReaderNext {
                word,
                children: children.clone(),
                remaining,
            }),
            Err(remaining) => {
                if let Some(next) = self.iterator.next() {
                    let value = next.value();
                    let reader = B256Reader::new(value);
                    self.current = Some((reader, next.children));
                } else {
                    let reader = B256Reader::new(B256::ZERO);
                    self.current = Some((reader, Default::default()));
                }
                self.next_with_remaining(remaining)
            }
        }
    }
}

impl<I> StorageReader for StorageReaderImpl<I>
where
    I: Iterator<Item = StorageNode>,
{
    fn next<'a, B: SubB256>(&mut self) -> Result<StorageReaderNext<B>, RemainingBytes> {
        self.next_with_remaining(RemainingBytes::default())
    }

    fn next_or_default<B: SubB256>(&mut self) -> StorageReaderNext<B> {
        match self.next::<B>() {
            Ok(next) => next,
            Err(remaining) => StorageReaderNext {
                word: Default::default(),
                children: Default::default(),
                remaining,
            },
        }
    }

    fn consume_remaining(&mut self) -> RemainingBytes {
        if let Some((reader, _)) = &mut self.current {
            let remaining_size = reader.remaining_size();
            if remaining_size == 32 || remaining_size == 0 {
                Bytes::default();
            }

            reader.consume_remaining()
        } else {
            RemainingBytes::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct B256Reader<const RIGHT_TO_LEFT: bool> {
    value: B256,
    index: usize,
}

impl<const RIGHT_TO_LEFT: bool> B256Reader<RIGHT_TO_LEFT> {
    pub const fn new(value: B256) -> Self {
        Self { value, index: 32 }
    }

    pub fn next<B: SubB256>(&mut self) -> Result<B, RemainingBytes> {
        assert!(B::SIZE <= 32);

        if let Some(new_index) = self.index.checked_sub(B::SIZE) {
            let result_slice = if RIGHT_TO_LEFT {
                &self.value[new_index..self.index]
            } else {
                &self.value[(32 - self.index)..(32 - new_index)]
            };

            debug_assert_eq!(result_slice.len(), B::SIZE);

            let result = B::from_slice(result_slice);

            self.index = new_index;

            Ok(result)
        } else {
            Err(self.remaining())
        }
    }

    pub const fn remaining_size(&self) -> usize {
        self.index
    }

    pub fn remaining(&self) -> RemainingBytes {
        let remaining_slice = if RIGHT_TO_LEFT {
            &self.value[..self.index]
        } else {
            &self.value[32 - self.index..]
        };
        RemainingBytes(Bytes::copy_from_slice(remaining_slice))
    }

    pub fn consume_remaining(&mut self) -> RemainingBytes {
        let remaining = self.remaining();
        self.index = 0;
        remaining
    }
}

pub trait SubB256: Default {
    const SIZE: usize;

    /// # Panics
    ///
    /// If the length of `src` and the number of bytes in `Self` do not match.
    fn from_slice(slice: &[u8]) -> Self;
}

macro_rules! impl_subb256 {
    ($($size:expr),*) => {
        $(
            impl SubB256 for FixedBytes<$size> {
                const SIZE: usize = $size;

                fn from_slice(slice: &[u8]) -> Self {
                    Self::from_slice(slice)
                }
            }
        )*
    };
}

impl_subb256!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32
);

#[cfg(test)]
mod tests {
    use super::*;

    use alloy_primitives::{b256, fixed_bytes};

    #[test]
    fn test_b256_reader_r2l() {
        let mut reader = B256Reader::<true>::new(b256!(
            "0x17d59586f58ca950436ec70e2b81075732ed8144283b3eb8c1a53be0f4e81b9f"
        ));

        assert_eq!(reader.next::<FixedBytes<1>>().unwrap(), fixed_bytes!("9f"));
        assert_eq!(
            reader.next::<FixedBytes<3>>().unwrap(),
            fixed_bytes!("f4e81b")
        );
        assert_eq!(
            reader.next::<FixedBytes<6>>().unwrap(),
            fixed_bytes!("3eb8c1a53be0")
        );
        assert_eq!(
            reader.next::<FixedBytes<20>>().unwrap(),
            fixed_bytes!("9586f58ca950436ec70e2b81075732ed8144283b")
        );
        assert!(reader.next::<FixedBytes<4>>().is_err());
        assert_eq!(
            reader.next::<FixedBytes<2>>().unwrap(),
            fixed_bytes!("17d5")
        );
    }

    #[test]
    fn test_b256_reader_l2r() {
        let mut reader = B256Reader::<false>::new(b256!(
            "0x17d59586f58ca950436ec70e2b81075732ed8144283b3eb8c1a53be0f4e81b9f"
        ));

        assert_eq!(reader.next::<FixedBytes<1>>().unwrap(), fixed_bytes!("17"));
        assert_eq!(
            reader.next::<FixedBytes<3>>().unwrap(),
            fixed_bytes!("d59586")
        );
        assert_eq!(
            reader.next::<FixedBytes<6>>().unwrap(),
            fixed_bytes!("f58ca950436e")
        );
        assert_eq!(
            reader.next::<FixedBytes<20>>().unwrap(),
            fixed_bytes!("c70e2b81075732ed8144283b3eb8c1a53be0f4e8")
        );
        assert!(reader.next::<FixedBytes<4>>().is_err());
        assert_eq!(
            reader.next::<FixedBytes<2>>().unwrap(),
            fixed_bytes!("1b9f")
        );
    }
}
