use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    interface IPool {}

    library DataTypes {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct ReserveDataLegacy {
            /// stores the reserve configuration
            ReserveConfigurationMap configuration;
            /// the liquidity index. Expressed in ray
            uint128 liquidityIndex;
            /// the current supply rate. Expressed in ray
            uint128 currentLiquidityRate;
            /// variable borrow index. Expressed in ray
            uint128 variableBorrowIndex;
            //the current variable borrow rate. Expressed in ray
            uint128 currentVariableBorrowRate;
            // DEPRECATED on v3.2.0
            uint128 currentStableBorrowRate;
            /// timestamp of last update
            uint40 lastUpdateTimestamp;
            //the id of the reserve. Represents the position in the list of the active reserves
            uint16 id;
            //aToken address
            address aTokenAddress;
            // DEPRECATED on v3.2.0
            address stableDebtTokenAddress;
            //variableDebtToken address
            address variableDebtTokenAddress;
            //address of the interest rate strategy
            address interestRateStrategyAddress;
            //the current treasury balance, scaled
            uint128 accruedToTreasury;
            //the outstanding unbacked aTokens minted through the bridging feature
            uint128 unbacked;
            //the outstanding debt borrowed against this asset in isolation mode
            uint128 isolationModeTotalDebt;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct ReserveData {
            //stores the reserve configuration
            ReserveConfigurationMap configuration;
            //the liquidity index. Expressed in ray
            uint128 liquidityIndex;
            //the current supply rate. Expressed in ray
            uint128 currentLiquidityRate;
            //variable borrow index. Expressed in ray
            uint128 variableBorrowIndex;
            //the current variable borrow rate. Expressed in ray
            uint128 currentVariableBorrowRate;
            /// @notice reused `__deprecatedStableBorrowRate` storage from pre 3.2
            // the current accumulate deficit in underlying tokens
            uint128 deficit;
            //timestamp of last update
            uint40 lastUpdateTimestamp;
            //the id of the reserve. Represents the position in the list of the active reserves
            uint16 id;
            //timestamp until when liquidations are not allowed on the reserve, if set to past liquidations will be allowed
            uint40 liquidationGracePeriodUntil;
            //aToken address
            address aTokenAddress;
            // DEPRECATED on v3.2.0
            address __deprecatedStableDebtTokenAddress;
            //variableDebtToken address
            address variableDebtTokenAddress;
            //address of the interest rate strategy
            address interestRateStrategyAddress;
            //the current treasury balance, scaled
            uint128 accruedToTreasury;
            //the outstanding unbacked aTokens minted through the bridging feature
            uint128 unbacked;
            //the outstanding debt borrowed against this asset in isolation mode
            uint128 isolationModeTotalDebt;
            //the amount of underlying accounted for by the protocol
            uint128 virtualUnderlyingBalance;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct ReserveConfigurationMap {
            //bit 0-15: LTV
            //bit 16-31: Liq. threshold
            //bit 32-47: Liq. bonus
            //bit 48-55: Decimals
            //bit 56: reserve is active
            //bit 57: reserve is frozen
            //bit 58: borrowing is enabled
            //bit 59: DEPRECATED: stable rate borrowing enabled
            //bit 60: asset is paused
            //bit 61: borrowing in isolation mode is enabled
            //bit 62: siloed borrowing enabled
            //bit 63: flashloaning enabled
            //bit 64-79: reserve factor
            //bit 80-115: borrow cap in whole tokens, borrowCap == 0 => no cap
            //bit 116-151: supply cap in whole tokens, supplyCap == 0 => no cap
            //bit 152-167: liquidation protocol fee
            //bit 168-175: DEPRECATED: eMode category
            //bit 176-211: unbacked mint cap in whole tokens, unbackedMintCap == 0 => minting disabled
            //bit 212-251: debt ceiling for isolation mode with (ReserveConfiguration::DEBT_CEILING_DECIMALS) decimals
            //bit 252: virtual accounting is enabled for the reserve
            //bit 253-255 unused

            uint256 data;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct UserConfigurationMap {
            /**
             * @dev Bitmap of the users collaterals and borrows. It is divided in pairs of bits, one pair per asset.
             * The first bit indicates if an asset is used as collateral by the user, the second whether an
             * asset is borrowed by the user.
             */
            uint256 data;
        }

        // DEPRECATED: kept for backwards compatibility, might be removed in a future version
        struct EModeCategoryLegacy {
            // each eMode category has a custom ltv and liquidation threshold
            uint16 ltv;
            uint16 liquidationThreshold;
            uint16 liquidationBonus;
            // DEPRECATED
            address priceSource;
            string label;
        }

        struct CollateralConfig {
            uint16 ltv;
            uint16 liquidationThreshold;
            uint16 liquidationBonus;
        }

        struct EModeCategoryBaseConfiguration {
            uint16 ltv;
            uint16 liquidationThreshold;
            uint16 liquidationBonus;
            string label;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct EModeCategory {
            // each eMode category has a custom ltv and liquidation threshold
            uint16 ltv;
            uint16 liquidationThreshold;
            uint16 liquidationBonus;
            uint128 collateralBitmap;
            string label;
            uint128 borrowableBitmap;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        enum InterestRateMode {
            NONE,
            __DEPRECATED,
            VARIABLE
        }

        struct ReserveCache {
            uint256 currScaledVariableDebt;
            uint256 nextScaledVariableDebt;
            uint256 currLiquidityIndex;
            uint256 nextLiquidityIndex;
            uint256 currVariableBorrowIndex;
            uint256 nextVariableBorrowIndex;
            uint256 currLiquidityRate;
            uint256 currVariableBorrowRate;
            uint256 reserveFactor;
            ReserveConfigurationMap reserveConfiguration;
            address aTokenAddress;
            address variableDebtTokenAddress;
            uint40 reserveLastUpdateTimestamp;
        }

        struct ExecuteLiquidationCallParams {
            uint256 reservesCount;
            uint256 debtToCover;
            address collateralAsset;
            address debtAsset;
            address user;
            bool receiveAToken;
            address priceOracle;
            uint8 userEModeCategory;
            address priceOracleSentinel;
        }

        struct ExecuteSupplyParams {
            address asset;
            uint256 amount;
            address onBehalfOf;
            uint16 referralCode;
        }

        struct ExecuteBorrowParams {
            address asset;
            address user;
            address onBehalfOf;
            uint256 amount;
            InterestRateMode interestRateMode;
            uint16 referralCode;
            bool releaseUnderlying;
            uint256 reservesCount;
            address oracle;
            uint8 userEModeCategory;
            address priceOracleSentinel;
        }

        struct ExecuteRepayParams {
            address asset;
            uint256 amount;
            InterestRateMode interestRateMode;
            address onBehalfOf;
            bool useATokens;
        }

        struct ExecuteWithdrawParams {
            address asset;
            uint256 amount;
            address to;
            uint256 reservesCount;
            address oracle;
            uint8 userEModeCategory;
        }

        struct ExecuteEliminateDeficitParams {
            address asset;
            uint256 amount;
        }

        struct ExecuteSetUserEModeParams {
            uint256 reservesCount;
            address oracle;
            uint8 categoryId;
        }

        struct FinalizeTransferParams {
            address asset;
            address from;
            address to;
            uint256 amount;
            uint256 balanceFromBefore;
            uint256 balanceToBefore;
            uint256 reservesCount;
            address oracle;
            uint8 fromEModeCategory;
        }

        struct FlashloanParams {
            address receiverAddress;
            address[] assets;
            uint256[] amounts;
            uint256[] interestRateModes;
            address onBehalfOf;
            bytes params;
            uint16 referralCode;
            uint256 flashLoanPremiumToProtocol;
            uint256 flashLoanPremiumTotal;
            uint256 reservesCount;
            address addressesProvider;
            address pool;
            uint8 userEModeCategory;
            bool isAuthorizedFlashBorrower;
        }

        struct FlashloanSimpleParams {
            address receiverAddress;
            address asset;
            uint256 amount;
            bytes params;
            uint16 referralCode;
            uint256 flashLoanPremiumToProtocol;
            uint256 flashLoanPremiumTotal;
        }

        struct FlashLoanRepaymentParams {
            uint256 amount;
            uint256 totalPremium;
            uint256 flashLoanPremiumToProtocol;
            address asset;
            address receiverAddress;
            uint16 referralCode;
        }

        struct CalculateUserAccountDataParams {
            UserConfigurationMap userConfig;
            uint256 reservesCount;
            address user;
            address oracle;
            uint8 userEModeCategory;
        }

        struct ValidateBorrowParams {
            ReserveCache reserveCache;
            UserConfigurationMap userConfig;
            address asset;
            address userAddress;
            uint256 amount;
            InterestRateMode interestRateMode;
            uint256 reservesCount;
            address oracle;
            uint8 userEModeCategory;
            address priceOracleSentinel;
            bool isolationModeActive;
            address isolationModeCollateralAddress;
            uint256 isolationModeDebtCeiling;
        }

        struct ValidateLiquidationCallParams {
            ReserveCache debtReserveCache;
            uint256 totalDebt;
            uint256 healthFactor;
            address priceOracleSentinel;
        }

        struct CalculateInterestRatesParams {
            uint256 unbacked;
            uint256 liquidityAdded;
            uint256 liquidityTaken;
            uint256 totalDebt;
            uint256 reserveFactor;
            address reserve;
            bool usingVirtualBalance;
            uint256 virtualUnderlyingBalance;
        }

        struct InitReserveParams {
            address asset;
            address aTokenAddress;
            address variableDebtAddress;
            address interestRateStrategyAddress;
            uint16 reservesCount;
            uint16 maxNumberReserves;
        }
    }

    abstract contract VersionedInitializable {
        /// Indicates that the contract has been initialized.
        uint256 private lastInitializedRevision = 0;

        /// Indicates that the contract is in the process of being initialized.
        bool private initializing;

        /// Reserved storage space to allow for layout changes in the future.
        uint256[50] private ______gap;
    }

    contract PoolStorage {
        /// Map of reserves and their data (underlyingAssetOfReserve => reserveData)
        mapping(address => DataTypes.ReserveData) internal _reserves;

        /// Map of users address and their configuration data (userAddress => userConfiguration)
        mapping(address => DataTypes.UserConfigurationMap) internal _usersConfig;

        /// List of reserves as a map (reserveId => reserve).
        /// It is structured as a mapping for gas savings reasons, using the reserve id as index
        mapping(uint256 => address) internal _reservesList;

        /// List of eMode categories as a map (eModeCategoryId => eModeCategory).
        /// It is structured as a mapping for gas savings reasons, using the eModeCategoryId as index
        mapping(uint8 => DataTypes.EModeCategory) internal _eModeCategories;

        /// Map of users address and their eMode category (userAddress => eModeCategoryId)
        mapping(address => uint8) internal _usersEModeCategory;

        /// Fee of the protocol bridge, expressed in bps
        uint256 internal _bridgeProtocolFee;

        /// Total FlashLoan Premium, expressed in bps
        uint128 internal _flashLoanPremiumTotal;

        /// FlashLoan premium paid to protocol treasury, expressed in bps
        uint128 internal _flashLoanPremiumToProtocol;

        /// DEPRECATED on v3.2.0
        uint64 internal __DEPRECATED_maxStableRateBorrowSizePercent;

        /// Maximum number of active reserves there have been in the protocol. It is the upper bound of the reserves list
        uint16 internal _reservesCount;
    }

    abstract contract Pool is VersionedInitializable, PoolStorage, IPool {
        IPoolAddressesProvider public immutable ADDRESSES_PROVIDER;

        bytes32 public constant UMBRELLA = "UMBRELLA";
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract PoolInstance is Pool {
        uint256 public constant POOL_REVISION = 7;

        #[sdecode(slot = "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc")]
        address public proxyImplem;
    }
}

fn main() {
    let path = "./test_data/aave-v3";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result =
        PoolInstanceStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract.decode::<PoolInstanceStorage>().unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
