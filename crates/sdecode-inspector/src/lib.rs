#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::{collections::BTreeSet, mem::take};

use alloy_primitives::{Address, B256, Bytes, U256};
use hashbrown::{HashMap, hash_map::Entry};
use overf::checked;
use quick_impl::{quick_impl, quick_impl_all};
use revm_bytecode::opcode::KECCAK256;
use revm_inspector::Inspector;
use revm_interpreter::{
    InstructionResult, Interpreter, InterpreterTypes, Stack,
    interpreter_types::{InputsTr, Jumps, LoopControl, MemoryTr, StackTr},
};
use sdecode_preimages::{Image, MemoryPreimagesProvider, Preimage};

/// Preimages inspector.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[quick_impl]
pub struct PreimagesInspector {
    unconfirmed: Option<(U256, U256)>,
    preimages: HashMap<Image, Preimage>,
    #[quick_impl(pub get = "{}", pub get_mut = "{}_mut", pub with, pub set)]
    targets: InspectorTargets,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
#[quick_impl_all(pub const is)]
pub enum InspectorTargets {
    #[default]
    All,
    #[quick_impl(pub as_ref, pub as_ref_mut, pub into, pub try_into)]
    Exclude(BTreeSet<Address>),
    #[quick_impl(pub as_ref, pub as_ref_mut, pub into, pub try_into)]
    IncludeOnly(BTreeSet<Address>),
}

impl InspectorTargets {
    pub fn should_inspect(&self, target: &Address) -> bool {
        match self {
            Self::All => true,
            Self::Exclude(targets) => !targets.contains(target),
            Self::IncludeOnly(targets) => targets.contains(target),
        }
    }

    pub fn should_not_inspect(&self, target: &Address) -> bool {
        match self {
            Self::All => false,
            Self::Exclude(targets) => targets.contains(target),
            Self::IncludeOnly(targets) => !targets.contains(target),
        }
    }

    pub fn exclude(targets: impl IntoIterator<Item = Address>) -> Self {
        Self::Exclude(targets.into_iter().collect())
    }

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
    /// Creates an empty [`PreimagesInspector`].
    pub fn new() -> Self {
        Self::new_with_target(InspectorTargets::All)
    }

    pub fn new_with_target(targets: InspectorTargets) -> Self {
        Self {
            unconfirmed: None,
            preimages: HashMap::new(),
            targets,
        }
    }

    pub fn new_excluding(targets: impl IntoIterator<Item = Address>) -> Self {
        Self::new_with_target(InspectorTargets::exclude(targets))
    }

    pub fn new_including_only(targets: impl IntoIterator<Item = Address>) -> Self {
        Self::new_with_target(InspectorTargets::include_only(targets))
    }

    /// Preimages reference.
    pub const fn preimages(&self) -> &HashMap<Image, Preimage> {
        &self.preimages
    }

    /// Take preimages.
    pub fn take_preimages(&mut self) -> HashMap<Image, Preimage> {
        take(&mut self.preimages)
    }

    /// Into preimages.
    pub fn into_preimages(self) -> HashMap<Image, Preimage> {
        self.preimages
    }

    pub fn into_provider(self) -> MemoryPreimagesProvider {
        MemoryPreimagesProvider::from_iter_unchecked(self.into_preimages())
    }
}

pub trait PeekableStack: StackTr {
    fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult>;
}

impl PeekableStack for Stack {
    fn peek(&self, no_from_top: usize) -> Result<U256, InstructionResult> {
        self.peek(no_from_top)
    }
}
