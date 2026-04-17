#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::collections::BTreeSet;

use alloy_primitives::{
    Address, B256, Bytes, U256,
    map::{B256Map, Entry},
};
use overf::checked;
use quick_impl::{quick_impl, quick_impl_all};
use revm_bytecode::opcode::KECCAK256;
use revm_inspector::Inspector;
use revm_interpreter::{
    InstructionResult, Interpreter, InterpreterTypes, Stack,
    interpreter_types::{InputsTr, Jumps, LoopControl, MemoryTr, StackTr},
};
use sdecode_preimages::{InMemoryPreimages, Preimage};

/// An EVM inspector that captures Keccak256 preimages during transaction execution.
///
/// This inspector hooks into the EVM execution to record the input data (preimage) for every
/// `KECCAK256` opcode. The collected preimages can then be converted into a
/// [`MemoryPreimagesProvider`] for later use in storage decoding.
///
/// Inspection can be scoped to specific contract addresses using [`InspectorTargets`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[quick_impl]
pub struct PreimagesInspector {
    unconfirmed: Option<(U256, U256)>,

    #[quick_impl(pub const get = "{}", pub take, pub into)]
    preimages: B256Map<Preimage>,

    #[quick_impl(pub const get = "{}", pub const get_mut = "{}_mut", pub with, pub set)]
    targets: InspectorTargets,
}

impl Default for PreimagesInspector {
    fn default() -> Self {
        Self::new()
    }
}

/// Controls which contract addresses should have their Keccak256 operations tracked.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[quick_impl_all(pub const is)]
pub enum InspectorTargets {
    /// Inspect all contracts. This is the default.
    #[default]
    All,

    /// Inspect all contracts except the ones in this set.
    #[quick_impl(pub as_ref, pub as_ref_mut, pub into, pub try_into)]
    Exclude(BTreeSet<Address>),

    /// Inspect only the contracts in this set.
    #[quick_impl(pub as_ref, pub as_ref_mut, pub into, pub try_into)]
    IncludeOnly(BTreeSet<Address>),
}

impl InspectorTargets {
    /// Returns `true` if the given address should be inspected.
    #[inline]
    #[must_use]
    pub fn should_inspect(&self, target: &Address) -> bool {
        match self {
            Self::All => true,
            Self::Exclude(targets) => !targets.contains(target),
            Self::IncludeOnly(targets) => targets.contains(target),
        }
    }

    /// Returns `true` if the given address should **not** be inspected.
    #[inline]
    #[must_use]
    pub fn should_not_inspect(&self, target: &Address) -> bool {
        match self {
            Self::All => false,
            Self::Exclude(targets) => targets.contains(target),
            Self::IncludeOnly(targets) => !targets.contains(target),
        }
    }

    /// Creates an [`Exclude`](Self::Exclude) target from the given addresses.
    #[must_use]
    pub fn exclude(targets: impl IntoIterator<Item = Address>) -> Self {
        Self::Exclude(targets.into_iter().collect())
    }

    /// Creates an [`IncludeOnly`](Self::IncludeOnly) target from the given addresses.
    #[must_use]
    pub fn include_only(targets: impl IntoIterator<Item = Address>) -> Self {
        Self::IncludeOnly(targets.into_iter().collect())
    }
}

impl<CTX, INTR> Inspector<CTX, INTR> for PreimagesInspector
where
    INTR: InterpreterTypes,
    INTR::Stack: PeekableStack,
{
    fn step(&mut self, interp: &mut Interpreter<INTR>, _: &mut CTX) {
        if interp.bytecode.opcode() != KECCAK256 {
            self.unconfirmed = None;
            return;
        }

        if self
            .targets
            .should_not_inspect(&interp.input.target_address())
        {
            self.unconfirmed = None;
            return;
        }

        let stack = &interp.stack;

        let offset = stack.peek(0).ok();
        let size = stack.peek(1).ok();

        self.unconfirmed = offset.zip(size);
    }

    fn step_end(&mut self, interp: &mut Interpreter<INTR>, _: &mut CTX) {
        let Some((offset, size)) = self.unconfirmed.take() else {
            return;
        };

        if interp
            .bytecode
            .instruction_result()
            .is_some_and(|instruction_result| !instruction_result.is_ok())
        {
            return;
        }

        let stack = &interp.stack;
        let image = B256::from(stack.peek(0).unwrap());

        if let Entry::Vacant(e) = self.preimages.entry(image) {
            let preimage = if size.is_zero() {
                Bytes::new()
            } else {
                let start = offset.to::<usize>();
                let end = checked! { start + size.to::<usize>() };
                let preimage_slice = interp.memory.slice(start..end);
                Bytes::copy_from_slice(preimage_slice.as_ref())
            };

            e.insert(preimage);
        }
    }
}

impl PreimagesInspector {
    /// Creates a new [`PreimagesInspector`] that inspects all contracts.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_target(InspectorTargets::All)
    }

    /// Creates a new [`PreimagesInspector`] with the given [`InspectorTargets`].
    #[must_use]
    pub fn new_with_target(targets: InspectorTargets) -> Self {
        Self {
            unconfirmed: None,
            preimages: Default::default(),
            targets,
        }
    }

    /// Clears the collected preimages.
    #[inline]
    pub fn clear(&mut self) {
        self.unconfirmed = None;
        self.preimages.clear();
    }

    /// Creates a new [`PreimagesInspector`] that inspects all contracts except the given ones.
    #[must_use]
    pub fn new_excluding(targets: impl IntoIterator<Item = Address>) -> Self {
        Self::new_with_target(InspectorTargets::exclude(targets))
    }

    /// Creates a new [`PreimagesInspector`] that only inspects the given contracts.
    #[must_use]
    pub fn new_including_only(targets: impl IntoIterator<Item = Address>) -> Self {
        Self::new_with_target(InspectorTargets::include_only(targets))
    }

    /// Consumes this inspector and returns a [`MemoryPreimagesProvider`] from the collected
    /// preimages.
    #[inline]
    #[must_use]
    pub fn into_provider(self) -> InMemoryPreimages {
        InMemoryPreimages::from_iter_unchecked(self.into_preimages())
    }
}

/// Trait for EVM stacks that support peeking at values without popping them.
pub trait PeekableStack: StackTr {
    /// Returns the value at `no_from_top` positions from the top of the stack.
    fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult>;
}

impl PeekableStack for Stack {
    #[inline]
    fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult> {
        self.peek(no_from_top)
    }
}
