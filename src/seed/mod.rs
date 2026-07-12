pub mod prng;
pub mod permute;
pub mod dialect;
pub mod crypto;
pub mod wordlists;
pub mod mnemonic;
pub mod shamir;

/// Canonical operation identifiers. This is the *fixed* vocabulary that the
/// glyph alphabet is permuted over. Every entry here maps to exactly one
/// glyph character in a given dialect (see [`crate::seed::permute`]).
///
/// Order does not matter (the permutation is seeded), only that the list is
/// stable across versions of the compiler for a given dialect id.
pub const CANONICAL_OPS: &[&str] = &[
    // literals / atoms
    "int_lit", "float_lit", "str_lit", "fstr_lit", "true_lit", "false_lit",
    "none_lit", "ellipsis_lit",
    // stack
    "load", "store", "dup", "drop", "swap", "over", "rot", "clear",
    // arithmetic
    "add", "sub", "mul", "div", "floordiv", "mod", "pow",
    // comparison
    "eq", "ne", "lt", "gt", "le", "ge", "in", "not_in", "is", "is_not",
    // logic / bitwise
    "and", "or", "not", "bitand", "bitor", "bitxor", "bitnot", "shl", "shr",
    // expressions
    "attr", "subscript", "slice", "call", "ternary", "walrus", "star", "dstar",
    "lambda", "comprehension", "await", "yield", "yield_from",
    // simple statements
    "aug_assign", "ann_assign", "unpack_assign", "del", "global", "nonlocal",
    "pass", "break", "continue", "ret", "raise", "assert",
    // compound statements
    "if", "elif", "else", "while", "for", "for_else", "try", "except",
    "try_else", "finally", "with", "match", "case", "async",
    // def / class
    "def", "def_ret", "decorator", "param_pos", "param_default", "param_slash",
    "param_kwonly", "param_args", "param_kwargs", "classdef",
    // oop extras
    "metaclass", "dataclass", "enum", "protocol", "slots", "property",
    "classmethod", "staticmethod",
    // imports
    "import", "from_import", "rel_import", "as_alias", "import_star",
    "main_guard",
    // match patterns
    "pat_lit", "pat_capture", "pat_wild", "pat_seq", "pat_map", "pat_class",
    "pat_guard", "pat_or",
    // builtins
    "print", "input", "len", "range", "enumerate", "zip", "sorted", "map",
    "filter", "reduce", "sum", "min", "max", "abs", "round", "reversed",
    "set_of", "list_of", "dict_of",
    // io
    "open", "read", "write", "readlines", "slurp", "dump",
    // macros
    "m_quicksort", "m_flask", "m_csv", "m_json", "m_fib", "m_argparse",
    "m_pytest", "m_ctxmgr", "m_pandas", "m_sqlalchemy", "m_regex",
    "m_threadpool", "m_asyncio",
    // structural
    "block_open", "block_close",
    // F1 compiler ops (used by the S-expression glyph source)
    "assign", "neg", "tuple",
];
