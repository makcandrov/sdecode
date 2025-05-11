use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    #[sdecode(language = "vyper")]
    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract CurveTricryptoOptimizedWETH {
        /// @nonreentrant decorator lock
        uint256 nonreentrant_status;

        address public immutable WETH20;

        uint256 constant N_COINS = 3;
        /// The precision to convert to.
        uint256 constant PRECISION = 10**8;
        uint256 constant A_MULTIPLIER = 10000;
        uint256 packed_precisions;

        Math public immutable MATH;
        address[N_COINS] public immutable coins;
        address public factory;

        /// Internal price scale.
        uint256 price_scale_packed;
        /// Price target given by moving average.
        uint256 price_oracle_packed;

        uint256 last_prices_packed;
        uint256 public last_prices_timestamp;

        uint256 public initial_A_gamma;
        uint256 public initial_A_gamma_time;

        uint256 public future_A_gamma;
        /// Time when ramping is finished.
        ///
        /// This value is 0 (default) when pool is first deployed, and only gets populated by block.timestamp + future_time in `ramp_A_gamma` when the block.timestamp + future_time in `ramp_A_gamma` when the ramping process is initiated. After ramping is finished (i.e. self.future_A_gamma_time < block.timestamp), the variable is left and not set to 0.
        uint256 public future_A_gamma_time;

        uint256[3] public balances;
        uint256 public D;
        uint256 public xcp_profit;
        /// Full profit at last claim of admin fees.
        uint256 public xcp_profit_a;

        /// Cached (fast to read) virtual price.
        ///
        /// The cached `virtual_price` is also used internally.
        uint256 public virtual_price;

        /// Contains rebalancing parameters allowed_extra_profit, adjustment_step, and ma_time.
        uint256 public packed_rebalancing_params;

        uint256 public future_packed_rebalancing_params;

        /// Packs mid_fee, out_fee, fee_gamma.
        uint256 public packed_fee_params;
        uint256 public future_packed_fee_params;

        /// 50% of earned fees.
        uint256 public constant ADMIN_FEE = 5 * 10**9;
        /// 0.5 BPS.
        uint256 constant MIN_FEE = 5 * 10**5;
        uint256 constant MAX_FEE = 10 * 10**9;
        /// 0.1 BPS.
        uint256 constant NOISE_FEE = 10**5;

        uint256 public admin_actions_deadline;

        uint256 constant ADMIN_ACTIONS_DELAY = 3 * 86400;
        uint256 constant MIN_RAMP_TIME = 86400;

        uint256 constant MIN_A = N_COINS**N_COINS * A_MULTIPLIER / 100;
        uint256 constant MAX_A = 1000 * A_MULTIPLIER * N_COINS**N_COINS;
        uint256 constant MAX_A_CHANGE = 10;
        uint256 constant MIN_GAMMA = 10**10;
        uint256 constant MAX_GAMMA = 5 * 10**16;

        uint128 constant PRICE_SIZE = 256 / (N_COINS - 1);
        uint256 constant PRICE_MASK = 2**PRICE_SIZE - 1;

        string immutable public name;
        string immutable public symbol;
        uint8 constant public decimals = 18;
        String constant public version = "v2.0.0";

        mapping(address => uint256) public balanceOf;
        mapping(address => mapping(address => uint256)) public allowance;
        uint256 public totalSupply;
        mapping(address => uint256) public nonces;

        bytes32 constant EIP712_TYPEHASH = keccak256(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)"
        );
        bytes32 constant EIP2612_TYPEHASH = keccak256(
            "Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)"
        );
        bytes32 constant  VERSION_HASH = keccak256(version);
        bytes32 immutable NAME_HASH;
        uint256 immutable CACHED_CHAIN_ID;
        bytes32 immutable salt;
        bytes32 immutable CACHED_DOMAIN_SEPARATOR;
    }
}

fn main() {
    let path = "./test_data/curve-tricrypto";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result =
        CurveTricryptoOptimizedWETHStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract
        .decode::<CurveTricryptoOptimizedWETHStorage>()
        .unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
