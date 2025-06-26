#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::{collections::BTreeSet, mem::replace, ops::Deref};

use alloy_primitives::{Address, B256, Bytes, U256};
use hashbrown::HashMap;
use overf::checked;
use revm_bytecode::opcode::KECCAK256;
use revm_inspector::Inspector;
use revm_interpreter::{
    InstructionResult, Interpreter, InterpreterTypes, Stack,
    interpreter_types::{InputsTr, Jumps, LoopControl, MemoryTr, StackTr},
};
use sdecode_preimages::{Image, MemoryPreimagesProvider, Preimage};

/// Preimages inspector.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PreimagesInspector {
    unconfirmed: Option<(U256, U256)>,
    preimages: HashMap<Image, Preimage>,
    targets: BTreeSet<Address>,
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

        if !self.targets.is_empty() && !self.targets.contains(&interp.input.target_address()) {
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

        if !interp
            .bytecode
            .instruction_result()
            .is_some_and(|instruction_result| instruction_result.is_ok())
        {
            return;
        }

        let stack = &interp.stack;
        let image = B256::from(stack.peek(0).unwrap());

        if let hashbrown::hash_map::Entry::Vacant(e) = self.preimages.entry(image) {
            let start = offset.to::<usize>();
            let end = checked! { start + size.to::<usize>() };
            let preimage_slice = interp.memory.slice(start..end);
            let preimage = Bytes::copy_from_slice(preimage_slice.deref());
            e.insert(preimage);
        }
    }
}

impl PreimagesInspector {
    /// Creates an empty [`PreimagesInspector`].
    pub fn new() -> Self {
        Self {
            unconfirmed: None,
            preimages: HashMap::new(),
            targets: BTreeSet::new(),
        }
    }

    pub fn new_with_target(target: Address) -> Self {
        Self::new().with_target(target)
    }

    pub fn new_with_targets(targets: impl IntoIterator<Item = Address>) -> Self {
        Self::new().with_targets(targets)
    }

    pub fn with_target(mut self, target: Address) -> Self {
        self.add_target(target);
        self
    }

    pub fn with_targets(mut self, targets: impl IntoIterator<Item = Address>) -> Self {
        self.add_targets(targets);
        self
    }

    pub fn add_target(&mut self, target: Address) {
        self.targets.insert(target);
    }

    pub fn add_targets(&mut self, targets: impl IntoIterator<Item = Address>) {
        self.targets.extend(targets);
    }

    /// Preimages reference.
    pub const fn preimages(&self) -> &HashMap<Image, Preimage> {
        &self.preimages
    }

    /// Take preimages.
    pub fn take_preimages(&mut self) -> HashMap<Image, Preimage> {
        replace(&mut self.preimages, HashMap::new())
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
