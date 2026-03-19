use std::collections::BTreeMap;

use alloy_primitives::{Address, B256, Bytes, U256, b256, bytes};
use sdecode_core::{IntoStorageReader, StorageNode, StorageStructure};
use sdecode_solidity::{SolLayoutError, SolStorageValue, sol_storage, sol_type};

/// Helper: decode a `SolStorageValue` from a single `StorageNode`.
fn decode_single<T, SolT>(node: StorageNode) -> Result<T, SolLayoutError>
where
    T: SolStorageValue<SolT>,
    SolT: sdecode_solidity::SolStorageType,
{
    let structure = StorageStructure::single_node(node);
    let mut reader = structure.into_storage_reader();
    T::decode_storage(&mut reader)
}

// ---------------------------------------------------------------------------
// RemainingBytes
// ---------------------------------------------------------------------------

#[test]
fn error_remaining_bytes() {
    sol_storage! {
        /// A struct where `address` (20 bytes) and `uint256` (32 bytes) live in
        /// consecutive slots. After reading the address from slot 0 (rightmost 20
        /// bytes), the leftmost 12 bytes remain. When the reader moves to slot 1
        /// for `uint256`, those 12 bytes are carried over as "remaining". If they
        /// are non-zero the decode fails with `RemainingBytes`.
        #[derive(Debug)]
        struct Packed {
            address addr;
            uint256 value;
        }
    }

    // Struct `{ address addr; uint256 value; }`
    //
    // Slot 0 holds the address in its rightmost 20 bytes. The leftmost 12
    // bytes are unused and must be zero. When they aren't, decoding the next
    // field (`uint256` from slot 1) fails because the reader carries over
    // those 12 dirty bytes as "remaining".
    let slot0 = b256!("0x000000000000000000deadff0000000000000000000000000000000012345678");
    //                  ^^^^^^^^^^^^^^^^^^^^^^^^ 12 bytes — non-zero!
    let slot1 = b256!("0x00000000000000000000000000000000000000000000000000000000000000ff");

    let nodes = vec![StorageNode::word(slot0), StorageNode::word(slot1)];
    let structure = StorageStructure(nodes);
    let mut reader = structure.into_storage_reader();

    let err = <Packed as SolStorageValue<Packed>>::decode_storage(&mut reader).unwrap_err();

    assert!(
        matches!(err, SolLayoutError::RemainingBytes { .. }),
        "expected RemainingBytes, got: {err}"
    );
    println!(
        "RemainingBytes (struct field crosses slot boundary with dirty leftover bytes):\n  {err}\n"
    );
}

#[test]
fn error_invalid_mapping_key() {
    // A mapping(address => uint256) expects 32-byte keys decodable as addresses.
    // Provide a child whose key is only 5 bytes — not a valid address encoding.
    let bad_key = bytes!("abcdef0102");
    let value_node = StorageStructure::single_node(StorageNode::word(b256!(
        "0x0000000000000000000000000000000000000000000000000000000000000042"
    )));

    let mut children = BTreeMap::new();
    children.insert(bad_key, value_node);

    let node = StorageNode {
        value: None,
        children,
    };

    let err =
        decode_single::<BTreeMap<Address, U256>, sol_type!(mapping(address => uint256))>(node)
            .unwrap_err();

    assert!(
        matches!(err, SolLayoutError::InvalidMappingKey { .. }),
        "expected InvalidMappingKey, got: {err}"
    );
    println!("InvalidMappingKey (5-byte key for address mapping):\n  {err}\n");
}

#[test]
fn error_non_empty_slot_mapping() {
    // A mapping slot must be zero. Put a non-zero word in it.
    let word = b256!("0x0000000000000000000000000000000000000000000000000000000000000001");
    let err = decode_single::<BTreeMap<Address, U256>, sol_type!(mapping(address => uint256))>(
        StorageNode::word(word),
    )
    .unwrap_err();

    assert!(
        matches!(err, SolLayoutError::NonEmptySlot { .. }),
        "expected NonEmptySlot, got: {err}"
    );
    println!("NonEmptySlot (mapping slot contains a non-zero value):\n  {err}\n");
}

#[test]
fn error_unexpected_children_on_word() {
    // A uint256 is a leaf — it should have no children.
    // Attach a child to simulate mismatched layout.
    let child_structure = StorageStructure::single_node(StorageNode::word(B256::ZERO));
    let mut children = BTreeMap::new();
    children.insert(bytes!("66616b655f6b6579"), child_structure); // "fake_key"

    let node = StorageNode {
        value: Some(b256!(
            "0x0000000000000000000000000000000000000000000000000000000000000042"
        )),
        children,
    };

    let err = decode_single::<U256, sol_type!(uint256)>(node).unwrap_err();

    assert!(
        matches!(err, SolLayoutError::UnexpectedChildren { count: 1 }),
        "expected UnexpectedChildren {{ count: 1 }}, got: {err}"
    );
    println!("UnexpectedChildren (uint256 slot has child entries):\n  {err}\n");
}

#[test]
fn error_unexpected_children_on_short_bytes() {
    // Short bytes (< 32 bytes) should have no children.
    // Word encodes short bytes: length=4 stored as last byte = 4*2 = 8.
    // Content "abcd" (4 bytes) in upper bytes, last byte = 0x08.
    let word = b256!("0x6162636400000000000000000000000000000000000000000000000000000008");

    let child_structure = StorageStructure::single_node(StorageNode::word(B256::ZERO));
    let mut children = BTreeMap::new();
    children.insert(Bytes::new(), child_structure);

    let node = StorageNode {
        value: Some(word),
        children,
    };

    let err = decode_single::<Vec<u8>, sol_type!(bytes)>(node).unwrap_err();

    assert!(
        matches!(err, SolLayoutError::UnexpectedChildren { .. }),
        "expected UnexpectedChildren, got: {err}"
    );
    println!("UnexpectedChildren (short bytes/string with child entries):\n  {err}\n");
}

#[test]
fn error_unexpected_children_on_dynamic_array() {
    // A dynamic array stores its length in the word and elements in children
    // under the empty key. Extra children with non-empty keys are an error.
    let length_word = b256!("0x0000000000000000000000000000000000000000000000000000000000000001");

    let element_structure = StorageStructure::single_node(StorageNode::word(b256!(
        "0x0000000000000000000000000000000000000000000000000000000000000042"
    )));

    let extra_structure = StorageStructure::single_node(StorageNode::word(B256::ZERO));

    let mut children = BTreeMap::new();
    children.insert(Bytes::new(), element_structure);
    children.insert(bytes!("756e65787065637465640a"), extra_structure); // "unexpected"

    let node = StorageNode {
        value: Some(length_word),
        children,
    };

    let err = decode_single::<Vec<U256>, sol_type!(uint256[])>(node).unwrap_err();

    assert!(
        matches!(err, SolLayoutError::UnexpectedChildren { .. }),
        "expected UnexpectedChildren, got: {err}"
    );
    println!("UnexpectedChildren (dynamic array with extra child keys):\n  {err}\n");
}

#[test]
fn error_array_length_overflow() {
    // A dynamic array length that exceeds u64::MAX.
    let huge_length = b256!("0x0000000000000001ffffffffffffffffffffffffffffffffffffffffffffffff");

    let node = StorageNode {
        value: Some(huge_length),
        children: BTreeMap::new(),
    };

    let err = decode_single::<Vec<U256>, sol_type!(uint256[])>(node).unwrap_err();

    assert!(
        matches!(err, SolLayoutError::ArrayLengthOverflow { .. }),
        "expected ArrayLengthOverflow, got: {err}"
    );
    println!("ArrayLengthOverflow (array claims impossibly large length):\n  {err}\n");
}

#[test]
fn error_invalid_word_value() {
    // Construct the error directly — this variant is a safety net for types
    // where the raw packed bytes fail to decode (e.g. a custom SolWordType).
    let err = SolLayoutError::InvalidWordValue { word: bytes!("02") };

    println!("InvalidWordValue (packed word bytes cannot be decoded):\n  {err}\n");

    // Also verify Display and Debug produce readable output.
    let display = format!("{err}");
    assert!(
        display.contains("invalid packed word value"),
        "unexpected display: {display}"
    );
}

#[test]
fn error_invalid_enum_discriminant() {
    sol_storage! {
        #[derive(Debug)]
        enum Status {
            First,
            Second,
            Third,
        }
    }

    // The Status enum has 3 variants (0, 1, 2). Discriminant 5 is invalid.
    let word = b256!("0x0000000000000000000000000000000000000000000000000000000000000005");

    let err = decode_single::<Status, Status>(StorageNode::word(word)).unwrap_err();

    assert!(
        matches!(
            err,
            SolLayoutError::InvalidEnumDiscriminant { discriminant: 5 }
        ),
        "expected InvalidEnumDiscriminant {{ discriminant: 5 }}, got: {err}"
    );
    println!("InvalidEnumDiscriminant (Status enum with discriminant 5):\n  {err}\n");
}

#[test]
fn print_all_error_variants() {
    let errors: Vec<(&str, SolLayoutError)> = vec![
        (
            "Struct field crosses slot boundary with dirty leftover bytes",
            SolLayoutError::RemainingBytes {
                remaining: bytes!("000000000000000000deadff"),
            },
        ),
        (
            "5-byte key used for address mapping",
            SolLayoutError::InvalidMappingKey {
                sol_type: "address",
                raw: bytes!("abcdef0102"),
            },
        ),
        (
            "Mapping slot is not empty",
            SolLayoutError::NonEmptySlot {
                sol_type: "mapping(address,uint256)",
                value: b256!("0x0000000000000000000000000000000000000000000000000000000000000001"),
            },
        ),
        (
            "uint256 slot has 3 unexpected child entries",
            SolLayoutError::UnexpectedChildren { count: 3 },
        ),
        (
            "Dynamic array claims 2^128 elements",
            SolLayoutError::ArrayLengthOverflow {
                value: b256!("0x0000000000000000000000000000000100000000000000000000000000000000"),
            },
        ),
        (
            "Bool slot contains 0x02",
            SolLayoutError::InvalidWordValue { word: bytes!("02") },
        ),
        (
            "Status enum got discriminant 5",
            SolLayoutError::InvalidEnumDiscriminant { discriminant: 5 },
        ),
    ];

    println!("=== All SolLayoutError variants ===\n");
    for (scenario, err) in &errors {
        println!("Scenario: {scenario}");
        println!("  Display: {err}");
        println!("  Debug:   {err:?}");
        println!();
    }
}
