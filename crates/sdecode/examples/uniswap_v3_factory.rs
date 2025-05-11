use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    interface IUniswapV3PoolDeployer {}
    interface IUniswapV3Factory {}

    abstract contract NoDelegateCall {
        /// @dev The original address of this contract
        address private immutable original;
    }

    contract UniswapV3PoolDeployer is IUniswapV3PoolDeployer {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Parameters {
            address factory;
            address token0;
            address token1;
            uint24 fee;
            int24 tickSpacing;
        }

        /// @inheritdoc IUniswapV3PoolDeployer
        Parameters public override parameters;
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract UniswapV3Factory is IUniswapV3Factory, UniswapV3PoolDeployer, NoDelegateCall {
        /// @inheritdoc IUniswapV3Factory
        address public override owner;

        /// @inheritdoc IUniswapV3Factory
        mapping(uint24 => int24) public override feeAmountTickSpacing;
        /// @inheritdoc IUniswapV3Factory
        mapping(address => mapping(address => mapping(uint24 => address))) public override getPool;
    }
}

fn main() {
    let path = "./test_data/uniswap-v3-factory";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result =
        UniswapV3FactoryStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract.decode::<UniswapV3FactoryStorage>().unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
