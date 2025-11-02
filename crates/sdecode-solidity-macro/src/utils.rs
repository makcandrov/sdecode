#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// ``, `20`
    Unknown,
    /// `erc`, `erc20`, `Erc`, `Erc20`
    Lower,
    /// `E`
    SingleUpper,
    /// `ER[C]`, `ERC20[U]`
    Upper(char),
    /// ERC20
    UpperUnknownEnd,
}

impl Case {
    pub fn try_from_char(ch: char) -> Option<Self> {
        if ch.is_lowercase() {
            Some(Self::Lower)
        } else if ch.is_uppercase() {
            Some(Self::Upper)
        } else {
            None
        }
    }
}

pub fn to_mod_name(s: &str) -> String {
    let mut res = String::with_capacity(s.len());

    let mut state = State::Unknown;

    for ch in s.chars() {
        let case = Case::try_from_char(ch);

        match (state, case) {
            (State::Unknown, Some(Case::Upper)) => {
                res.push(ch.to_ascii_lowercase());
                state = State::SingleUpper;
            }
            (State::Unknown, Some(Case::Lower)) => {
                res.push(ch);
                state = State::Lower;
            }
            (State::Lower, Some(Case::Upper)) => {
                res.push('_');
                res.push(ch.to_ascii_lowercase());
                state = State::SingleUpper;
            }
            (State::SingleUpper, Some(Case::Lower)) => {
                res.push(ch);
                state = State::Lower;
            }
            (State::SingleUpper, Some(Case::Upper)) => {
                state = State::Upper(ch);
            }
            (State::SingleUpper, None) => {
                res.push(ch.to_ascii_lowercase());
                state = State::UpperUnknownEnd;
            }
            (State::Upper(last), None) => {
                res.push(last.to_ascii_lowercase());
                res.push(ch.to_ascii_lowercase());
                state = State::UpperUnknownEnd;
            }
            (State::Upper(last), Some(Case::Upper)) => {
                res.push(last.to_ascii_lowercase());
                state = State::Upper(ch);
            }
            (State::Upper(last), Some(Case::Lower)) => {
                res.push('_');
                res.push(last.to_ascii_lowercase());
                res.push(ch);
                state = State::Lower;
            }
            (State::UpperUnknownEnd, Some(Case::Lower)) => {
                res.push('_');
                res.push(ch);
                state = State::Lower;
            }
            (State::UpperUnknownEnd, Some(Case::Upper)) => {
                state = State::Upper(ch);
            }
            _ => {
                res.push(ch.to_ascii_lowercase());
            }
        }
    }

    if let State::Upper(ch) = state {
        res.push(ch.to_ascii_lowercase());
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_mod_name() {
        fn test(input: &str, output: &str) {
            assert_eq!(to_mod_name(input), output);
        }

        test("", "");
        test("A", "a");
        test("ABC", "abc");
        test("ERC20", "erc20");
        test("ERC20Upgradeable", "erc20_upgradeable");
        test("NFTXVaultUpgradeable", "nftx_vault_upgradeable");
        test("IERC1271", "ierc1271");
        test(
            "IERC3156FlashLenderUpgradeable",
            "ierc3156_flash_lender_upgradeable",
        );
        test("UniswapV3Factory", "uniswap_v3_factory");
        test("IUniswapV3Factory", "i_uniswap_v3_factory");
        test(
            "CurveTricryptoOptimizedWETH",
            "curve_tricrypto_optimized_weth",
        );
        test("Abc12defGhi345klmN", "abc12def_ghi345klm_n");
        test("20erc", "20erc");
        test("ERC20erc20", "erc20_erc20");
    }
}
