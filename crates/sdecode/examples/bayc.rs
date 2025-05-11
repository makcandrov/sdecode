use std::time::Instant;

use sdecode::solidity::sol_storage;
use sdecode_test_utils::{JsonUtils, SdecodeTestContract};

sol_storage! {
    interface IERC165 {}
    interface IERC721Enumerable {}
    interface IERC721Metadata {}
    interface IERC721 {}

    library EnumerableSet {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Set {
            /// Storage of set values
            bytes32[] _values;

            /// Position of the value in the `values` array, plus 1 because index 0
            /// means a value is not in the set.
            mapping (bytes32 => uint256) _indexes;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Bytes32Set {
            Set _inner;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct AddressSet {
            Set _inner;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct UintSet {
            Set _inner;
        }
    }

    library EnumerableMap {
        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct MapEntry {
            bytes32 _key;
            bytes32 _value;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct Map {
            /// Storage of map keys and values
            MapEntry[] _entries;

            /// Position of the entry defined by a key in the `entries` array, plus 1
            /// because index 0 means a key is not in the map.
            mapping (bytes32 => uint256) _indexes;
        }

        #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        struct UintToAddressMap {
            Map _inner;
        }
    }

    abstract contract Context {}

    abstract contract Ownable is Context {
        address private _owner;
    }

    abstract contract ERC165 is IERC165 {
        /*
         * bytes4(keccak256('supportsInterface(bytes4)')) == 0x01ffc9a7
         */
        bytes4 private constant _INTERFACE_ID_ERC165 = 0x01ffc9a7;

        /**
         * @dev Mapping of interface ids to whether or not it's supported.
         */
        mapping(bytes4 => bool) private _supportedInterfaces;
    }

    contract ERC721 is Context, ERC165, IERC721, IERC721Metadata, IERC721Enumerable {
        bytes4 private constant _ERC721_RECEIVED = 0x150b7a02;

        /// Mapping from holder address to their (enumerable) set of owned tokens
        mapping (address => EnumerableSet.UintSet) private _holderTokens;

        /// Enumerable mapping from token ids to their owners
        EnumerableMap.UintToAddressMap private _tokenOwners;

        /// Mapping from token ID to approved address
        mapping (uint256 => address) private _tokenApprovals;

        /// Mapping from owner to operator approvals
        mapping (address => mapping (address => bool)) private _operatorApprovals;

        /// Token name
        string private _name;

        /// Token symbol
        string private _symbol;

        /// Optional mapping for token URIs
        mapping (uint256 => string) private _tokenURIs;

        /// Base URI
        string private _baseURI;

        bytes4 private constant _INTERFACE_ID_ERC721 = 0x80ac58cd;
        bytes4 private constant _INTERFACE_ID_ERC721_METADATA = 0x5b5e139f;
        bytes4 private constant _INTERFACE_ID_ERC721_ENUMERABLE = 0x780e9d63;
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    contract BoredApeYachtClub is ERC721, Ownable {
        string public BAYC_PROVENANCE = "";

        uint256 public startingIndexBlock;

        uint256 public startingIndex;

        uint256 public constant apePrice = 80000000000000000; //0.08 ETH

        uint public constant maxApePurchase = 20;

        uint256 public MAX_APES;

        bool public saleIsActive = false;

        uint256 public REVEAL_TIMESTAMP;
    }
}

fn main() {
    let path = "./test_data/bayc";

    let contract = SdecodeTestContract::from_json_file(format!("{path}/input.json")).unwrap();
    let expected_result =
        BoredApeYachtClubStorage::from_json_file(format!("{path}/output.json")).unwrap();

    println!(
        "decoding contract addr={} block={} chain_id={}",
        contract.address, contract.block, contract.chain_id
    );

    let now = Instant::now();
    let decoded = contract.decode::<BoredApeYachtClubStorage>().unwrap();
    let elapsed = now.elapsed();

    assert_eq!(expected_result, decoded);

    println!("decoded contract in {:?}", elapsed);
}
