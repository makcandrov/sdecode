use alloy_primitives::{B256, Bytes, TxKind, address, bytes, keccak256, map::B256Map};
use revm::{
    Context, InspectEvm, MainBuilder, MainContext,
    context::{BlockEnv, TxEnv},
};
use sdecode_inspector::PreimagesInspector;

#[test]
fn test_preimages_inspector() {
    let mut insp = PreimagesInspector::default();
    let mut evm = Context::mainnet()
        .with_block(BlockEnv::default())
        .build_mainnet()
        .with_inspector(&mut insp);

    evm.inspect_tx(TxEnv {
        caller: address!("0x1212000000000000000000000000000000000000"),
        kind: TxKind::Create,
        // push(32) push(0) keccak256 push(0) push(0) return
        data: bytes!("0x60205f205f5ff3"),
        ..Default::default()
    })
    .unwrap();

    let mut map = B256Map::default();
    map.insert(keccak256(B256::ZERO), Bytes::from(B256::ZERO));

    assert_eq!(insp.into_preimages(), map,);
}
