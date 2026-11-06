use alloy_consensus::{TxLegacy, transaction::Recovered};
use alloy_primitives::{Address, Bytes, TxKind, U256, address, keccak256, map::B256Map};
use evm_asm::bytecode;
use evm2::{
    AccountInfo, BaseEvmTypes, Evm, EvmTypesHost, Inspector, Precompiles, SpecId, TxResult,
    bytecode::Bytecode,
    ethereum::{TxEnvelope, ethereum_tx_registry},
    evm::InMemoryDB,
    interpreter::{InstrStop, Interpreter, op},
};
use sdecode_inspector::{InspectorTargets, PreimagesInspector};

const CALLER: Address = address!("0x1212000000000000000000000000000000000000");
const ENTRY: Address = address!("0x1000000000000000000000000000000000000000");
const CHILD: Address = address!("0x2000000000000000000000000000000000000000");

const PARENT_HASHING_AROUND_CALL: &[u8] = bytecode! {
    push1 0xaa push0 mstore8
    push1 1 push0 keccak256 pop
    push0 push0 push0 push0 push0 push20 #(CHILD.0.0) gas call pop
    push1 2 push0 keccak256
};
const CHILD_HASHING: &[u8] = bytecode! {
    push1 0xbb push0 mstore8
    push1 1 push0 keccak256
};

fn transact(
    accounts: &[(Address, &[u8])],
    to: TxKind,
    input: impl Into<Bytes>,
    inspector: &mut impl Inspector<BaseEvmTypes>,
) -> TxResult<BaseEvmTypes> {
    let mut db = InMemoryDB::default();
    for (address, code) in accounts {
        let code = Bytecode::new_legacy(Bytes::copy_from_slice(code));
        db.insert_account_info(address, AccountInfo::default().with_code(code));
    }

    let spec = SpecId::OSAKA;
    let mut evm = Evm::<BaseEvmTypes>::new(
        spec,
        Default::default(),
        ethereum_tx_registry(spec),
        db,
        Precompiles::base(spec),
    );

    let tx = Recovered::new_unchecked(
        TxEnvelope::Legacy(TxLegacy {
            gas_limit: 1_000_000,
            to,
            input: input.into(),
            ..Default::default()
        }),
        CALLER,
    );

    evm.transact_with_inspector(&tx, inspector)
        .unwrap()
        .discard()
}

fn call_entry(
    accounts: &[(Address, &[u8])],
    targets: InspectorTargets,
) -> (TxResult<BaseEvmTypes>, B256Map<Bytes>) {
    let mut insp = PreimagesInspector::new_with_target(targets);
    let result = transact(accounts, TxKind::Call(ENTRY), Bytes::new(), &mut insp);
    (result, insp.into_preimages())
}

fn run(code: &[u8]) -> (TxResult<BaseEvmTypes>, B256Map<Bytes>) {
    call_entry(&[(ENTRY, code)], InspectorTargets::All)
}

fn preimages<'a>(preimages: impl IntoIterator<Item = &'a [u8]>) -> B256Map<Bytes> {
    preimages
        .into_iter()
        .map(|preimage| (keccak256(preimage), Bytes::copy_from_slice(preimage)))
        .collect()
}

#[test]
fn records_preimage_in_create() {
    let mut insp = PreimagesInspector::default();
    let result = transact(
        &[],
        TxKind::Create,
        bytecode!(push1 32 push0 keccak256 push0 push0 return),
        &mut insp,
    );

    assert!(result.is_success());
    assert_eq!(insp.into_preimages(), preimages([&[0; 32][..]]));
}

#[test]
fn records_zero_size_with_out_of_range_offset() {
    let (result, recorded) = run(bytecode! {
        push0
        push32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
        keccak256
    });

    assert!(result.is_success());
    assert_eq!(recorded, preimages([&[][..]]));
}

#[test]
fn reads_memory_expanded_by_keccak256() {
    let (result, recorded) = run(bytecode!(push1 64 push0 keccak256));

    assert!(result.is_success());
    assert_eq!(recorded, preimages([&[0; 64][..]]));
}

#[test]
fn records_unaligned_region() {
    let (result, recorded) = run(bytecode! {
        push32 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
        push0 mstore
        push1 33 push1 1 keccak256
    });

    let expected: Vec<u8> = (2..=32).chain([0, 0]).collect();

    assert!(result.is_success());
    assert_eq!(recorded, preimages([&expected[..]]));
}

#[test]
fn ignores_halted_keccak256() {
    let cases: [(&str, &[u8], InstrStop); 6] = [
        (
            "offset out of range",
            bytecode! {
                push1 32
                push32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                keccak256
            },
            InstrStop::InvalidOperandOOG,
        ),
        (
            "size out of range",
            bytecode! {
                push32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                push0
                keccak256
            },
            InstrStop::InvalidOperandOOG,
        ),
        (
            "offset + size overflows",
            bytecode!(push1 32 push8 0xffffffffffffffff keccak256),
            InstrStop::MemoryOOG,
        ),
        (
            "memory expansion too expensive",
            bytecode!(push4 0x01000000 push0 keccak256),
            InstrStop::OutOfGas,
        ),
        (
            "one stack item",
            bytecode!(push1 32 keccak256),
            InstrStop::StackUnderflow,
        ),
        (
            "empty stack",
            bytecode!(keccak256),
            InstrStop::StackUnderflow,
        ),
    ];

    for (name, code, stop) in cases {
        let (result, recorded) = run(code);
        assert_eq!(result.stop, stop, "{name}");
        assert!(recorded.is_empty(), "{name}");
    }
}

#[test]
fn ignores_keccak256_out_of_gas() {
    let parent: &[u8] = bytecode! {
        push0 push0 push0 push0 push0 push20 #(CHILD.0.0) push0 calldataload call
    };
    let child: &[u8] = bytecode!(push1 32 push0 keccak256);

    // The child spends 5 gas on pushes, then KECCAK256 costs 30 static + 6 per word + 3 memory.
    for (gas, expected) in [
        (34, B256Map::default()),
        (43, B256Map::default()),
        (44, preimages([&[0; 32][..]])),
    ] {
        let mut insp = PreimagesInspector::default();
        let result = transact(
            &[(ENTRY, parent), (CHILD, child)],
            TxKind::Call(ENTRY),
            U256::from(gas).to_be_bytes_vec(),
            &mut insp,
        );

        assert!(result.is_success(), "gas {gas}");
        assert_eq!(insp.into_preimages(), expected, "gas {gas}");
    }
}

#[test]
fn records_preimage_of_reverted_execution() {
    let (result, recorded) = run(bytecode!(push1 32 push0 keccak256 push0 push0 revert));

    assert_eq!(result.stop, InstrStop::Revert);
    assert_eq!(recorded, preimages([&[0; 32][..]]));
}

#[test]
fn uses_frame_memory_across_calls() {
    let expected = preimages([&[0xaa][..], &[0xbb], &[0xaa, 0]]);
    let accounts = [(ENTRY, PARENT_HASHING_AROUND_CALL), (CHILD, CHILD_HASHING)];

    let (result, recorded) = call_entry(&accounts, InspectorTargets::All);
    assert!(result.is_success());
    assert_eq!(recorded, expected);

    let mut insp = (ObserveAll, PreimagesInspector::default());
    let result = transact(&accounts, TxKind::Call(ENTRY), Bytes::new(), &mut insp);
    assert!(result.is_success());
    assert_eq!(insp.1.into_preimages(), expected);
}

#[test]
fn filters_targets() {
    let accounts = [(ENTRY, PARENT_HASHING_AROUND_CALL), (CHILD, CHILD_HASHING)];

    let (_, recorded) = call_entry(&accounts, InspectorTargets::include_only([CHILD]));
    assert_eq!(recorded, preimages([&[0xbb][..]]));

    let (_, recorded) = call_entry(&accounts, InspectorTargets::exclude([CHILD]));
    assert_eq!(recorded, preimages([&[0xaa][..], &[0xaa, 0]]));
}

#[test]
fn attributes_delegatecall_to_caller() {
    let proxy: &[u8] = bytecode! {
        push0 push0 push0 push0 push20 #(CHILD.0.0) gas delegatecall
    };
    let accounts = [(ENTRY, proxy), (CHILD, CHILD_HASHING)];

    let (result, recorded) = call_entry(&accounts, InspectorTargets::include_only([ENTRY]));
    assert!(result.is_success());
    assert_eq!(recorded, preimages([&[0xbb][..]]));

    let (_, recorded) = call_entry(&accounts, InspectorTargets::include_only([CHILD]));
    assert!(recorded.is_empty());
}

#[test]
fn ignores_keccak256_halted_by_other_inspector_in_step() {
    let calling: &[u8] = bytecode! {
        push1 0xaa push0 mstore8
        push0 push0 push0 push0 push0 push20 #(CHILD.0.0) gas call
        push1 1 push0 keccak256
    };
    let creating: &[u8] = bytecode! {
        push8 #CHILD_HASHING push0 mstore
        push1 8 push1 24 push0 create
        push1 0xaa push0 mstore8
        push1 1 push0 keccak256
    };
    let expected = preimages([&[0xaa][..]]);

    for (name, parent) in [("call", calling), ("create", creating)] {
        let accounts = [(ENTRY, parent), (CHILD, CHILD_HASHING)];

        let mut insp = (HaltKeccak256Outside(ENTRY), PreimagesInspector::default());
        let result = transact(&accounts, TxKind::Call(ENTRY), Bytes::new(), &mut insp);
        assert!(result.is_success(), "{name}");
        assert_eq!(insp.1.into_preimages(), expected, "{name}");

        let mut insp = (PreimagesInspector::default(), HaltKeccak256Outside(ENTRY));
        let result = transact(&accounts, TxKind::Call(ENTRY), Bytes::new(), &mut insp);
        assert!(result.is_success(), "{name}");
        assert_eq!(insp.0.into_preimages(), expected, "{name}");
    }
}

struct ObserveAll;

impl<T: EvmTypesHost> Inspector<T> for ObserveAll {}

struct HaltKeccak256Outside(Address);

impl<T: EvmTypesHost> Inspector<T> for HaltKeccak256Outside {
    fn step(&mut self, interp: &mut Interpreter<'_, '_, T>) {
        if interp.opcode() == op::KECCAK256 && interp.message().destination != self.0 {
            interp.set_stop(InstrStop::Revert);
        }
    }
}
