use std::collections::{BTreeMap, btree_map};

use alloy_primitives::{B256, Bytes};
use quick_impl::quick_impl;

use crate::{HashChain, HashLink, IntoStorageReader, StorageReader, StorageReaderImpl};

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct StorageNode {
    pub value: Option<B256>,
    pub children: StorageNodeChildren,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[quick_impl]
pub struct StorageStructure(#[quick_impl(impl Deref, impl From, impl Into)] pub Vec<StorageNode>);

pub type StorageNodeChildren = BTreeMap<Bytes, StorageStructure>;

impl StorageNode {
    pub const fn empty() -> Self {
        Self {
            value: None,
            children: BTreeMap::new(),
        }
    }

    pub const fn word(value: B256) -> Self {
        Self {
            value: Some(value),
            children: BTreeMap::new(),
        }
    }

    pub fn single_child(key: Bytes, child: StorageStructure) -> Self {
        let mut children = BTreeMap::new();
        children.insert(key, child);
        Self {
            value: None,
            children,
        }
    }

    pub fn with_child(mut self, key: Bytes, child: StorageStructure) -> Self {
        self.children.insert(key, child);
        self
    }

    pub fn from_link(link: HashLink) -> Self {
        match link {
            HashLink::Leaf { value } => Self::word(value),
            HashLink::Inner {
                key,
                remaining_chain: child,
            } => {
                let mut children = BTreeMap::new();
                children.insert(key, StorageStructure::from_chain(*child));
                Self {
                    value: None,
                    children,
                }
            }
        }
    }

    pub fn value(&self) -> B256 {
        self.value.unwrap_or_default()
    }

    pub fn add_link(&mut self, link: HashLink) {
        match link {
            HashLink::Leaf { value } => {
                let old = self.value.replace(value);

                // panic or error?
                assert!(old.is_none_or(|old_value| old_value == value));
            }
            HashLink::Inner {
                key,
                remaining_chain,
            } => match self.children.entry(key) {
                btree_map::Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(StorageStructure::from_chain(*remaining_chain));
                }
                btree_map::Entry::Occupied(mut occupied_entry) => {
                    occupied_entry.get_mut().add_chain(*remaining_chain);
                }
            },
        }
    }
}

impl StorageStructure {
    pub fn single_node(node: StorageNode) -> Self {
        Self(vec![node])
    }

    pub fn from_chain(chain: HashChain) -> Self {
        let mut nodes = vec![StorageNode::empty(); chain.offset];
        let last_node = StorageNode::from_link(chain.link);
        nodes.push(last_node);
        Self(nodes)
    }

    pub fn add_chain(&mut self, chain: HashChain) {
        if let Some(delta) = chain.offset.checked_sub(self.0.len()) {
            self.0.extend(vec![StorageNode::empty(); delta]);
            self.0.push(StorageNode::from_link(chain.link));
        } else {
            let node = &mut self.0[chain.offset];
            node.add_link(chain.link);
        }
    }
}

impl FromIterator<StorageNode> for StorageStructure {
    fn from_iter<T: IntoIterator<Item = StorageNode>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl FromIterator<B256> for StorageStructure {
    fn from_iter<T: IntoIterator<Item = B256>>(iter: T) -> Self {
        iter.into_iter().map(StorageNode::word).collect()
    }
}

impl<'a> FromIterator<&'a B256> for StorageStructure {
    fn from_iter<T: IntoIterator<Item = &'a B256>>(iter: T) -> Self {
        iter.into_iter().copied().collect()
    }
}

impl IntoStorageReader for StorageStructure {
    fn into_storage_reader(self) -> impl StorageReader {
        StorageReaderImpl::new(self.0.into_iter())
    }
}
