use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    interface IExttload {}
    interface IExtsload {}
    interface IERC6909Claims {}
    interface IProtocolFees {}
    interface IPoolManager {}

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
    type PoolId is bytes32;
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
    type Currency is address;
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
    type Slot0 is bytes32;

    abstract contract Exttload is IExttload {}
    abstract contract Extsload is IExtsload {}

    abstract contract ERC6909 is IERC6909Claims {
        mapping(address owner => mapping(address operator => bool isOperator)) public isOperator;

        mapping(address owner => mapping(uint256 id => uint256 balance)) public balanceOf;

        mapping(address owner => mapping(address spender => mapping(uint256 id => uint256 amount))) public allowance;
    }

    abstract contract ERC6909Claims is ERC6909 {}

    abstract contract NoDelegateCall {
        error DelegateCallNotAllowed();

        /// The original address of this contract
        address private immutable original;
    }

    abstract contract Owned {
        event OwnershipTransferred(address indexed user, address indexed newOwner);

        address public owner;
    }

    abstract contract ProtocolFees is IProtocolFees, Owned {
        /// @inheritdoc IProtocolFees
        mapping(Currency currency => uint256 amount) public protocolFeesAccrued;

        /// @inheritdoc IProtocolFees
        address public protocolFeeController;
    }

    library Position {
        /// info stored for each user's position
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct State {
            /// the amount of liquidity owned by this position
            uint128 liquidity;
            /// fee growth per unit of liquidity as of the last update to liquidity or fees owed
            uint256 feeGrowthInside0LastX128;
            uint256 feeGrowthInside1LastX128;
        }
    }

    library Pool {
        /// info stored for each initialized individual tick
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct TickInfo {
            /// the total position liquidity that references this tick
            uint128 liquidityGross;
            /// amount of net liquidity added (subtracted) when tick is crossed from left to right (right to left),
            int128 liquidityNet;
            /// fee growth per unit of liquidity on the _other_ side of this tick (relative to the current tick)
            /// only has relative meaning, not absolute — the value depends on when the tick is initialized
            uint256 feeGrowthOutside0X128;
            uint256 feeGrowthOutside1X128;
        }

        /// @notice The state of a pool
        /// @dev Note that feeGrowthGlobal can be artificially inflated
        /// For pools with a single liquidity position, actors can donate to themselves to freely inflate feeGrowthGlobal
        /// atomically donating and collecting fees in the same unlockCallback may make the inflated value more extreme
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct State {
            Slot0 slot0;
            uint256 feeGrowthGlobal0X128;
            uint256 feeGrowthGlobal1X128;
            uint128 liquidity;
            mapping(int24 tick => TickInfo) ticks;
            mapping(int16 wordPos => uint256) tickBitmap;
            mapping(bytes32 positionKey => Position.State) positions;
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract PoolManager is IPoolManager, ProtocolFees, NoDelegateCall, ERC6909Claims, Extsload, Exttload {
        int24 private constant MAX_TICK_SPACING = TickMath.MAX_TICK_SPACING;

        int24 private constant MIN_TICK_SPACING = TickMath.MIN_TICK_SPACING;

        mapping(PoolId id => Pool.State) internal _pools;
    }
}

fn main() {
    let path = "./test_data/uniswap-v4";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result =
        PoolManagerStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract.decode::<PoolManagerStorage>().unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
