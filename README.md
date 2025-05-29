# sdecode

Decode any EVM storage.

```rust
use alloy_primitives::address;
use sdecode::{PreimagesProvider, StorageDecode, StorageEntries, solidity::sol_storage};

sol_storage! {
    contract WETH {
        string public name;
        string public symbol;
        uint8 public decimals;

        mapping (address => uint) public balanceOf;
        mapping (address => mapping (address => uint)) public allowance;
    }
}

fn decode_weth_storage<P: PreimagesProvider>(
    preimages_provider: P,
    storage_entries: StorageEntries,
) {
    let weth_storage = WETHStorage::sdecode(preimages_provider, storage_entries).unwrap();

    assert_eq!(&weth_storage.name, "Wrapped Ether");
    assert_eq!(&weth_storage.symbol, "WETH");
    assert_eq!(weth_storage.decimals, 18u8);

    let holders_number = weth_storage.balanceOf.len();

    let vitalik_balance = weth_storage
        .balanceOf
        .get(&address!("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .cloned()
        .unwrap_or_default();
}
```

Runnable examples for some major Ethereum contracts can be found in [the examples folder](./crates/sdecode/examples/).

The `sol_storage!` macro takes a Solidity contract as input and generates its storage structure. It implements the `StorageDecode` trait, which provides the decoding method `sdecode`.

To accurately decode storage, two elements are necessary:
- All the storage entries of the contract, as a mapping of slot to value.
- A Keccak256 preimages database containing at least all the preimages necessary for decoding.

## Full Storage

Currently, there is no official API in major Ethereum nodes to query the full storage of a contract. You will need to implement this manually in your node. An adapter to query it from Reth’s database may be added to this repository in the future.

## Preimages

Since mappings and other dynamically sized data structures store their values at the Keccak256 hash of their slot concatenated with some key, reversing this process is impossible if the preimage is unknown. This is why a preimages database implementing the `PreimagesProvider` trait is required. The database can be built by tracing each transaction that interacts with the contract using the inspector provided in [`sdecode-inspector`](./crates/sdecode-inspector/).

## Vyper support

This crate does not support directly embedding Vyper code in the `sol_storage!` macro. That's because `sol_storage!` relies on [`syn-solidity`] to parse Solidity syntax, and no equivalent parser exists for Vyper at the moment. However, you can manually translate a Vyper contract into Solidity syntax, then annotate it with `#[sdecode(language = "vyper")]`. Be careful with Vyper-specific behavior. For example, the `@nonreentrant` decorator inserts a hidden storage slot at the beginning of the layout.

For a complete example of decoding a Vyper contract, see [the Curve Tricrypto pool example](./crates/sdecode//examples/curve_tricrypto.rs).

[`syn-solidity`]: https://crates.io/crates/syn-solidity
