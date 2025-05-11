use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    interface IMorphoStaticTyping {}

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
    type Id is bytes32;

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct MarketParams {
        address loanToken;
        address collateralToken;
        address oracle;
        address irm;
        uint256 lltv;
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Position {
        uint256 supplyShares;
        uint128 borrowShares;
        uint128 collateral;
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct Market {
        uint128 totalSupplyAssets;
        uint128 totalSupplyShares;
        uint128 totalBorrowAssets;
        uint128 totalBorrowShares;
        uint128 lastUpdate;
        uint128 fee;
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract Morpho is IMorphoStaticTyping {
        /// @inheritdoc IMorphoBase
        bytes32 public immutable DOMAIN_SEPARATOR;

        /// @inheritdoc IMorphoBase
        address public owner;
        /// @inheritdoc IMorphoBase
        address public feeRecipient;
        /// @inheritdoc IMorphoStaticTyping
        mapping(Id => mapping(address => Position)) public position;
        /// @inheritdoc IMorphoStaticTyping
        mapping(Id => Market) public market;
        /// @inheritdoc IMorphoBase
        mapping(address => bool) public isIrmEnabled;
        /// @inheritdoc IMorphoBase
        mapping(uint256 => bool) public isLltvEnabled;
        /// @inheritdoc IMorphoBase
        mapping(address => mapping(address => bool)) public isAuthorized;
        /// @inheritdoc IMorphoBase
        mapping(address => uint256) public nonce;
        /// @inheritdoc IMorphoStaticTyping
        mapping(Id => MarketParams) public idToMarketParams;
    }
}

fn main() {
    let path = "./test_data/morpho-blue";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result = MorphoStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract.decode::<MorphoStorage>().unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
