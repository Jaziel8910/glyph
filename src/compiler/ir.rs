//! Intermediate representation for Glyph F1. This is the language-agnostic AST
//! that both the glyph S-expression parser and the Python front-end produce,
//! and that the Python emitter consumes. Keep it small and explicit.

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub name: Option<String>,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Lit),
    Name(String),
    Bin {
        op: BinOp,
        l: Box<Expr>,
        r: Box<Expr>,
    },
    Un {
        op: UnOp,
        e: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Arg>,
    },
    Attr {
        obj: Box<Expr>,
        name: String,
    },
    Sub {
        obj: Box<Expr>,
        idx: Box<Expr>,
    },
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Ternary {
        test: Box<Expr>,
        then: Box<Expr>,
        els: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Normal,
    Star,
    DoubleStar,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub kind: ParamKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Assign {
        target: Expr,
        value: Expr,
    },
    Aug {
        op: BinOp,
        target: Expr,
        value: Expr,
    },
    Def {
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    Ret(Option<Expr>),
    /// Branches in order. A branch with `test == None` is the `else` clause.
    If {
        branches: Vec<(Option<Expr>, Vec<Stmt>)>,
    },
    While {
        test: Expr,
        body: Vec<Stmt>,
    },
    For {
        target: Expr,
        iter: Expr,
        body: Vec<Stmt>,
    },
    Import {
        names: Vec<(String, Option<String>)>,
    },
    FromImport {
        module: String,
        names: Vec<(String, Option<String>)>,
    },
    Break,
    Continue,
    Pass,
}
