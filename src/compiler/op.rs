//! Operator tables shared by the glyph and Python codegen. The glyph dialect
//! maps each *operator name* (a string also present in `CANONICAL_OPS`) to a
//! single glyph char; these tables describe the *shape* of each operator so the
//! parser knows how to read/emit it.

use crate::compiler::ir::{BinOp, UnOp};

/// Map a canonical op name to a binary operator, if it is one.
pub fn binop_from_name(name: &str) -> Option<BinOp> {
    Some(match name {
        "add" => BinOp::Add,
        "sub" => BinOp::Sub,
        "mul" => BinOp::Mul,
        "div" => BinOp::Div,
        "floordiv" => BinOp::FloorDiv,
        "mod" => BinOp::Mod,
        "pow" => BinOp::Pow,
        "eq" => BinOp::Eq,
        "ne" => BinOp::Ne,
        "lt" => BinOp::Lt,
        "gt" => BinOp::Gt,
        "le" => BinOp::Le,
        "ge" => BinOp::Ge,
        "and" => BinOp::And,
        "or" => BinOp::Or,
        _ => return None,
    })
}

/// Python source symbol for a binary operator.
pub fn binop_symbol(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::FloorDiv => "//",
        BinOp::Mod => "%",
        BinOp::Pow => "**",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Gt => ">",
        BinOp::Le => "<=",
        BinOp::Ge => ">=",
        BinOp::And => "and",
        BinOp::Or => "or",
    }
}

pub fn unop_symbol(op: UnOp) -> &'static str {
    match op {
        UnOp::Not => "not",
        UnOp::Neg => "-",
    }
}

/// Nullary statement operators (standalone glyph char, no arguments).
pub fn is_nullary_stmt(name: &str) -> bool {
    matches!(name, "break" | "continue" | "pass")
}

/// Statement-forming operators (head of a parenthesized form).
pub fn is_stmt_form(name: &str) -> bool {
    matches!(
        name,
        "def" | "ret" | "if" | "while" | "for" | "assign" | "aug" | "import" | "from_import"
    )
}

/// Expression-forming operators (head of a parenthesized form).
pub fn is_expr_form(name: &str) -> bool {
    binop_from_name(name).is_some()
        || matches!(
            name,
            "not" | "neg" | "call" | "attr" | "subscript" | "list_of" | "tuple" | "dict_of"
                | "ternary"
        )
}
