use syn_solidity::{BinOp, Expr, Lit, LitNumber, Spanned, UnOp};

use crate::{
    pp::UserDefinedItem,
    scope::{Scope, Scoped},
};

pub type ArraySize = usize;

pub struct ArraySizeEvaluator {}

impl ArraySizeEvaluator {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn eval(&self, sc: &Scope<'_>, expr: &Expr) -> syn::Result<ArraySize> {
        match expr {
            Expr::Lit(Lit::Number(LitNumber::Int(n))) => n
                .base10_digits()
                .parse::<ArraySize>()
                .map_err(|_| syn::Error::new(expr.span(), "unable to determine array length")),
            Expr::Binary(bin) => {
                let lhs = self.eval(sc, &bin.left)?;
                let rhs = self.eval(sc, &bin.right)?;
                self.eval_binop(bin.op, lhs, rhs)
            }
            Expr::Unary(unary) => {
                let value = self.eval(sc, &unary.expr)?;
                self.eval_unop(unary.op, value)
            }
            Expr::Ident(ident) => match sc.user_defined_item_ident(&ident.0) {
                Some(Scoped {
                    inner: UserDefinedItem::Variable(var),
                    scope,
                }) => {
                    if !var.raw.attributes.has_constant() {
                        return Err(syn::Error::new(ident.span(), "non constant variable"));
                    }

                    let Some((_, initializer)) = &var.raw.initializer else {
                        return Err(syn::Error::new(
                            ident.span(),
                            "variable without initializer",
                        ));
                    };

                    self.eval(&scope, initializer)
                }
                _ => Err(syn::Error::new(ident.span(), "expected constant variable")),
            },
            _ => Err(syn::Error::new(
                expr.span(),
                "unable to determine array length",
            )),
        }
    }

    fn eval_binop(&self, bin: BinOp, lhs: ArraySize, rhs: ArraySize) -> syn::Result<ArraySize> {
        let result = match bin {
            BinOp::Shr(..) => rhs.try_into().ok().and_then(|rhs| lhs.checked_shr(rhs)),
            BinOp::Shl(..) => rhs.try_into().ok().and_then(|rhs| lhs.checked_shl(rhs)),
            BinOp::BitAnd(..) => Some(lhs & rhs),
            BinOp::BitOr(..) => Some(lhs | rhs),
            BinOp::BitXor(..) => Some(lhs ^ rhs),
            BinOp::Add(..) => lhs.checked_add(rhs),
            BinOp::Sub(..) => lhs.checked_sub(rhs),
            BinOp::Pow(..) => rhs.try_into().ok().and_then(|rhs| lhs.checked_pow(rhs)),
            BinOp::Mul(..) => lhs.checked_mul(rhs),
            BinOp::Div(..) => lhs.checked_div(rhs),
            BinOp::Rem(..) => lhs.checked_rem(rhs),
            _ => return Err(syn::Error::new(bin.span(), "operation not supported")),
        };

        result.ok_or_else(|| syn::Error::new(bin.span(), "arithmetic overflow"))
    }

    fn eval_unop(&self, unop: UnOp, value: ArraySize) -> syn::Result<ArraySize> {
        let result = match unop {
            UnOp::Neg(..) => value.checked_neg(),
            UnOp::BitNot(..) | UnOp::Not(..) => Some(!value),
            _ => return Err(syn::Error::new(unop.span(), "operation not supported")),
        };
        result.ok_or_else(|| syn::Error::new(unop.span(), "arithmetic overflow"))
    }
}
