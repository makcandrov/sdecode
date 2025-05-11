use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    interface ProtocolFeesCollector {}
    interface IAuthorizer {}
    interface IAuthentication {}
    interface ISignaturesValidator {}
    interface ITemporarilyPausable {}
    interface IVault {}
    interface IERC20 {}

    library EnumerableMap {
        /// The original OpenZeppelin implementation uses a generic Map type with bytes32 keys: this was replaced with
        /// IERC20ToBytes32Map, which uses IERC20 keys natively, resulting in more dense bytecode.
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct IERC20ToBytes32MapEntry {
            IERC20 _key;
            bytes32 _value;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct IERC20ToBytes32Map {
            /// Number of entries in the map
            uint256 _length;
            /// Storage of map keys and values
            mapping(uint256 => IERC20ToBytes32MapEntry) _entries;
            /// Position of the entry defined by a key in the `entries` array, plus 1
            /// because index 0 means a key is not in the map.
            mapping(IERC20 => uint256) _indexes;
        }
    }

    library EnumerableSet {
        /// The original OpenZeppelin implementation uses a generic Set type with bytes32 values: this was replaced with
        /// AddressSet, which uses address keys natively, resulting in more dense bytecode.
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct AddressSet {
            /// Storage of set values
            address[] _values;
            /// Position of the value in the `values` array, plus 1 because index 0
            /// means a value is not in the set.
            mapping(address => uint256) _indexes;
        }
    }

    abstract contract ReentrancyGuard {
        uint256 private constant _NOT_ENTERED = 1;
        uint256 private constant _ENTERED = 2;

        uint256 private _status;
    }

    abstract contract Authentication is IAuthentication {
        bytes32 private immutable _actionIdDisambiguator;
    }

    abstract contract EIP712 {
        bytes32 private immutable _HASHED_NAME;
        bytes32 private immutable _HASHED_VERSION;
        bytes32 private immutable _TYPE_HASH;
    }

    abstract contract SignaturesValidator is ISignaturesValidator, EIP712 {
        uint256 internal constant _EXTRA_CALLDATA_LENGTH = 4 * 32;

        /// Replay attack prevention for each user.
        mapping(address => uint256) internal _nextNonce;
    }

    abstract contract TemporarilyPausable is ITemporarilyPausable {
        uint256 private constant _MAX_PAUSE_WINDOW_DURATION = 90 days;
        uint256 private constant _MAX_BUFFER_PERIOD_DURATION = 30 days;

        uint256 private immutable _pauseWindowEndTime;
        uint256 private immutable _bufferPeriodEndTime;

        bool private _paused;
    }

    abstract contract Fees is IVault {
        ProtocolFeesCollector private immutable _protocolFeesCollector;
    }

    abstract contract FlashLoans is Fees, ReentrancyGuard, TemporarilyPausable {}

    abstract contract VaultAuthorization is
        IVault,
        ReentrancyGuard,
        Authentication,
        SignaturesValidator,
        TemporarilyPausable
    {
        bytes32 private constant _JOIN_TYPE_HASH = 0x3f7b71252bd19113ff48c19c6e004a9bcfcca320a0d74d58e85877cbd7dcae58;
        bytes32 private constant _EXIT_TYPE_HASH = 0x8bbc57f66ea936902f50a71ce12b92c43f3c5340bb40c27c4e90ab84eeae3353;
        bytes32 private constant _SWAP_TYPE_HASH = 0xe192dcbc143b1e244ad73b813fd3c097b832ad260a157340b4e5e5beda067abe;
        bytes32 private constant _BATCH_SWAP_TYPE_HASH = 0x9bfc43a4d98313c6766986ffd7c916c7481566d9f224c6819af0a53388aced3a;
        bytes32 private constant _SET_RELAYER_TYPE_HASH = 0xa3f865aa351e51cfeb40f5178d1564bb629fe9030b83caf6361d1baaf5b90b5a;

        IAuthorizer private _authorizer;
        mapping(address => mapping(address => bool)) private _approvedRelayers;
    }

    abstract contract PoolRegistry is ReentrancyGuard, VaultAuthorization {
        /// Each pool is represented by their unique Pool ID. We use `bytes32` for them, for lack of a way to define new
        /// types.
        mapping(bytes32 => bool) private _isPoolRegistered;

        /// We keep an increasing nonce to make Pool IDs unique. It is interpreted as a `uint80`, but storing it as a
        /// `uint256` results in reduced bytecode on reads and writes due to the lack of masking.
        uint256 private _nextPoolNonce;
    }

    abstract contract GeneralPoolsBalance {
        /// Data for Pools with the General specialization setting
        ///
        /// These Pools use the IGeneralPool interface, which means the Vault must query the balance for *all* of their
        /// tokens in every swap. If we kept a mapping of token to balance plus a set (array) of tokens, it'd be very gas
        /// intensive to read all token addresses just to then do a lookup on the balance mapping.
        ///
        /// Instead, we use our customized EnumerableMap, which lets us read the N balances in N+1 storage accesses (one for
        /// each token in the Pool), access the index of any 'token in' a single read (required for the IGeneralPool call),
        /// and update an entry's value given its index.
        ///
        /// Map of token -> balance pairs for each Pool with this specialization. Many functions rely on storage pointers to
        /// a Pool's EnumerableMap to save gas when computing storage slots.
        mapping(bytes32 => EnumerableMap.IERC20ToBytes32Map) internal _generalPoolsBalances;
    }

    abstract contract MinimalSwapInfoPoolsBalance is PoolRegistry {
        /// Data for Pools with the Minimal Swap Info specialization setting
        ///
        /// These Pools use the IMinimalSwapInfoPool interface, and so the Vault must read the balance of the two tokens
        /// in the swap. The best solution is to use a mapping from token to balance, which lets us read or write any token's
        /// balance in a single storage access.
        ///
        /// We also keep a set of registered tokens. Because tokens with non-zero balance are by definition registered, in
        /// some balance getters we skip checking for token registration if a non-zero balance is found, saving gas by
        /// performing a single read instead of two.
        mapping(bytes32 => mapping(IERC20 => bytes32)) internal _minimalSwapInfoPoolsBalances;
        mapping(bytes32 => EnumerableSet.AddressSet) internal _minimalSwapInfoPoolsTokens;
    }

    abstract contract TwoTokenPoolsBalance is PoolRegistry {
        /// Data for Pools with the Two Token specialization setting
        ///
        /// These are similar to the Minimal Swap Info Pool case (because the Pool only has two tokens, and therefore there
        /// are only two balances to read), but there's a key difference in how data is stored. Keeping a set makes little
        /// sense, as it will only ever hold two tokens, so we can just store those two directly.
        ///
        /// The gas savings associated with using these Pools come from how token balances are stored: cash amounts for token
        /// A and token B are packed together, as are managed amounts. Because only cash changes in a swap, there's no need
        /// to write to this second storage slot. A single last change block number for both tokens is stored with the packed
        /// cash fields.
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct TwoTokenPoolBalances {
            bytes32 sharedCash;
            bytes32 sharedManaged;
        }

        /// We could just keep a mapping from Pool ID to TwoTokenSharedBalances, but there's an issue: we wouldn't know to
        /// which tokens those balances correspond. This would mean having to also check which are registered with the Pool.
        ///
        /// What we do instead to save those storage reads is keep a nested mapping from the token pair hash to the balances
        /// struct. The Pool only has two tokens, so only a single entry of this mapping is set (the one that corresponds to
        /// that pair's hash).
        ///
        /// This has the trade-off of making Vault code that interacts with these Pools cumbersome: both balances must be
        /// accessed at the same time by using both token addresses, and some logic is needed to determine how the pair hash
        /// is computed. We do this by sorting the tokens, calling the token with the lowest numerical address value token A,
        /// and the other one token B. In functions where the token arguments could be either A or B, we use X and Y instead.
        ///
        /// If users query a token pair containing an unregistered token, the Pool will generate a hash for a mapping entry
        /// that was not set, and return zero balances. Non-zero balances are only possible if both tokens in the pair
        // are registered with the Pool, which means we don't have to check the TwoTokenPoolTokens struct, and can save
        /// storage reads.
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct TwoTokenPoolTokens {
            IERC20 tokenA;
            IERC20 tokenB;
            mapping(bytes32 => TwoTokenPoolBalances) balances;
        }

        mapping(bytes32 => TwoTokenPoolTokens) private _twoTokenPoolTokens;
    }

    abstract contract AssetManagers is
        ReentrancyGuard,
        GeneralPoolsBalance,
        MinimalSwapInfoPoolsBalance,
        TwoTokenPoolsBalance
    {
        /// Stores the Asset Manager for each token of each Pool.
        mapping(bytes32 => mapping(IERC20 => address)) internal _poolAssetManagers;
    }

    abstract contract PoolTokens is ReentrancyGuard, PoolRegistry, AssetManagers {}

    abstract contract AssetHelpers {
        IWETH private immutable _weth;
        address private constant _ETH = address(0);
    }

    abstract contract AssetTransfersHandler is AssetHelpers {}

    abstract contract UserBalance is ReentrancyGuard, AssetTransfersHandler, VaultAuthorization {
        /// Internal Balance for each token, for each account.
        mapping(address => mapping(IERC20 => uint256)) private _internalTokenBalance;
    }

    abstract contract PoolBalances is Fees, ReentrancyGuard, PoolTokens, UserBalance {}

    abstract contract Swaps is ReentrancyGuard, PoolBalances {}

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract Vault is VaultAuthorization, FlashLoans, Swaps {}
}

fn main() {
    let path = "./test_data/balancer-v2";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result = VaultStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract.decode::<VaultStorage>().unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
