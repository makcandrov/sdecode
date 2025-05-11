use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    /// @title Tick
    /// @notice Contains functions for managing tick processes and relevant calculations
    library Tick {
        /// info stored for each initialized individual tick
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Info {
            /// the total position liquidity that references this tick
            uint128 liquidityGross;
            /// amount of net liquidity added (subtracted) when tick is crossed from left to right (right to left),
            int128 liquidityNet;
            /// fee growth per unit of liquidity on the _other_ side of this tick (relative to the current tick)
            /// only has relative meaning, not absolute — the value depends on when the tick is initialized
            uint256 feeGrowthOutside0X128;
            uint256 feeGrowthOutside1X128;
            /// the cumulative tick value on the other side of the tick
            int56 tickCumulativeOutside;
            /// the seconds per unit of liquidity on the _other_ side of this tick (relative to the current tick)
            /// only has relative meaning, not absolute — the value depends on when the tick is initialized
            uint160 secondsPerLiquidityOutsideX128;
            /// the seconds spent on the other side of the tick (relative to the current tick)
            /// only has relative meaning, not absolute — the value depends on when the tick is initialized
            uint32 secondsOutside;
            /// true iff the tick is initialized, i.e. the value is exactly equivalent to the expression liquidityGross != 0
            /// these 8 bits are set to prevent fresh sstores when crossing newly initialized ticks
            bool initialized;
        }
    }

    /// @title Position
    /// @notice Positions represent an owner address' liquidity between a lower and upper tick boundary
    /// @dev Positions store additional state for tracking fees owed to the position
    library Position {
        /// info stored for each user's position
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Info {
            /// the amount of liquidity owned by this position
            uint128 liquidity;
            /// fee growth per unit of liquidity as of the last update to liquidity or fees owed
            uint256 feeGrowthInside0LastX128;
            uint256 feeGrowthInside1LastX128;
            /// the fees owed to the position owner in token0/token1
            uint128 tokensOwed0;
            uint128 tokensOwed1;
        }
    }

    /// @title Oracle
    /// @notice Provides price and liquidity data useful for a wide variety of system designs
    /// @dev Instances of stored oracle data, "observations", are collected in the oracle array
    /// Every pool is initialized with an oracle array length of 1. Anyone can pay the SSTOREs to increase the
    /// maximum length of the oracle array. New slots will be added when the array is fully populated.
    /// Observations are overwritten when the full length of the oracle array is populated.
    /// The most recent observation is available, independent of the length of the oracle array, by passing 0 to observe()
    library Oracle {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Observation {
            /// the block timestamp of the observation
            uint32 blockTimestamp;
            /// the tick accumulator, i.e. tick * time elapsed since the pool was first initialized
            int56 tickCumulative;
            /// the seconds per liquidity, i.e. seconds elapsed / max(1, liquidity) since the pool was first initialized
            uint160 secondsPerLiquidityCumulativeX128;
            /// whether or not the observation is initialized
            bool initialized;
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract UniswapV3Pool {
        /// @inheritdoc IUniswapV3PoolImmutables
        address public immutable override factory;
        /// @inheritdoc IUniswapV3PoolImmutables
        address public immutable override token0;
        /// @inheritdoc IUniswapV3PoolImmutables
        address public immutable override token1;
        /// @inheritdoc IUniswapV3PoolImmutables
        uint24 public immutable override fee;

        /// @inheritdoc IUniswapV3PoolImmutables
        int24 public immutable override tickSpacing;

        /// @inheritdoc IUniswapV3PoolImmutables
        uint128 public immutable override maxLiquidityPerTick;

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Slot0 {
            // the current price
            uint160 sqrtPriceX96;
            // the current tick
            int24 tick;
            // the most-recently updated index of the observations array
            uint16 observationIndex;
            // the current maximum number of observations that are being stored
            uint16 observationCardinality;
            // the next maximum number of observations to store, triggered in observations.write
            uint16 observationCardinalityNext;
            // the current protocol fee as a percentage of the swap fee taken on withdrawal
            // represented as an integer denominator (1/x)%
            uint8 feeProtocol;
            // whether the pool is locked
            bool unlocked;
        }
        /// @inheritdoc IUniswapV3PoolState
        Slot0 public override slot0;

        /// @inheritdoc IUniswapV3PoolState
        uint256 public override feeGrowthGlobal0X128;
        /// @inheritdoc IUniswapV3PoolState
        uint256 public override feeGrowthGlobal1X128;

        /// accumulated protocol fees in token0/token1 units
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct ProtocolFees {
            uint128 token0;
            uint128 token1;
        }
        /// @inheritdoc IUniswapV3PoolState
        ProtocolFees public override protocolFees;

        /// @inheritdoc IUniswapV3PoolState
        uint128 public override liquidity;

        /// @inheritdoc IUniswapV3PoolState
        mapping(int24 => Tick.Info) public override ticks;
        /// @inheritdoc IUniswapV3PoolState
        mapping(int16 => uint256) public override tickBitmap;
        /// @inheritdoc IUniswapV3PoolState
        mapping(bytes32 => Position.Info) public override positions;
        /// @inheritdoc IUniswapV3PoolState
        Oracle.Observation[65535] public override observations;
    }
}

fn main() {
    let path = "./test_data/uniswap-v3-pool";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result =
        UniswapV3PoolStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract.decode::<UniswapV3PoolStorage>().unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
