use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    interface IERC1271 {}
    interface INFTXVault {}
    interface IERC165Upgradeable {}
    interface IERC1155ReceiverUpgradeable is IERC165Upgradeable {}
    interface IERC721ReceiverUpgradeable {}
    interface IERC3156FlashLenderUpgradeable {}
    interface IERC20Upgradeable {}
    interface IERC20Metadata is IERC20Upgradeable {}
    interface INFTXVaultFactory {}
    interface INFTXEligibility {}

    library EnumerableSetUpgradeable {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Set {
            /// Storage of set values
            bytes32[] _values;
            /// Position of the value in the `values` array, plus 1 because index 0
            /// means a value is not in the set.
            mapping(bytes32 => uint256) _indexes;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct UintSet {
            Set _inner;
        }
    }

    abstract contract ERC165Upgradeable is IERC165Upgradeable {}

    abstract contract ERC1155ReceiverUpgradeable is
        ERC165Upgradeable,
        IERC1155ReceiverUpgradeable
    {}

    abstract contract ERC1155SafeHolderUpgradeable is ERC1155ReceiverUpgradeable {}

    contract ERC721SafeHolderUpgradeable is IERC721ReceiverUpgradeable {}

    abstract contract Initializable {
        ///Indicates that the contract has been initialized.
        bool private _initialized;

        ///Indicates that the contract is in the process of being initialized.
        bool private _initializing;
    }

    abstract contract ReentrancyGuardUpgradeable is Initializable {
        uint256 private constant _NOT_ENTERED = 1;
        uint256 private constant _ENTERED = 2;

        uint256 private _status;

        uint256[49] private __gap_ReentrancyGuardUpgradeable;
    }

    abstract contract ContextUpgradeable is Initializable {
        uint256[50] private __gap_ContextUpgradeable;
    }

    contract ERC20Upgradeable is
        Initializable,
        ContextUpgradeable,
        IERC20Upgradeable,
        IERC20Metadata
    {
        mapping(address => uint256) private _balances;

        mapping(address => mapping(address => uint256)) private _allowances;

        uint256 private _totalSupply;

        string private _name;
        string private _symbol;

        uint256[45] private __gap_ERC20Upgradeable;
    }

    abstract contract ERC20FlashMintUpgradeable is
        Initializable,
        ERC20Upgradeable,
        IERC3156FlashLenderUpgradeable
    {
        uint256[50] private __gap_ERC20FlashMintUpgradeable;
    }

    abstract contract OwnableUpgradeable is Initializable, ContextUpgradeable {
        address private _owner;
        uint256[49] private __gap_OwnableUpgradeable;
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract NFTXVaultUpgradeable is
        OwnableUpgradeable,
        ERC20FlashMintUpgradeable,
        ReentrancyGuardUpgradeable,
        ERC721SafeHolderUpgradeable,
        ERC1155SafeHolderUpgradeable,
        INFTXVault,
        IERC1271
    {
        using EnumerableSetUpgradeable for EnumerableSetUpgradeable.UintSet;

        uint256 constant base = 10 ** 18;

        uint256 public override vaultId;
        address public override manager;
        address public override assetAddress;
        INFTXVaultFactory public override vaultFactory;
        INFTXEligibility public override eligibilityStorage;

        uint256 randNonce;
        uint256 private UNUSED_FEE1;
        uint256 private UNUSED_FEE2;
        uint256 private UNUSED_FEE3;

        bool public override is1155;
        bool public override allowAllItems;
        bool public override enableMint;
        bool public override enableRandomRedeem;
        bool public override enableTargetRedeem;

        EnumerableSetUpgradeable.UintSet holdings;
        mapping(uint256 => uint256) quantity1155;

        bool public override enableRandomSwap;
        bool public override enableTargetSwap;

        #[sdecode(slot = "0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50")]
        address proxyImplem;
    }
}

fn main() {
    let path = "./test_data/punk";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result =
        NFTXVaultUpgradeableStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract.decode::<NFTXVaultUpgradeableStorage>().unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
