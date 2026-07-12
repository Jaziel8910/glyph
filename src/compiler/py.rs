//! Python-subset front-end and emitter.
//!
//! `parse_python` turns a *subset* of Python into the IR; `emit_python` turns IR
//! back into valid Python. Together with the glyph codegen this gives a full
//! round-trip: `.py` -> IR -> `.glf` (encrypted) -> IR -> `.py`.
//!
//! Supported subset: defs, assignments (+ augmented), if/elif/else, while, for,
//! return, break/continue/pass, import/from-import (with `as`), expressions
//! (literals, names, binops with precedence, `not`, calls, attribute access,
//! subscript, lists, tuples, dicts, ternary). Not supported (F1): classes,
//! decorators, comprehensions, async, lambdas, slicing, f-strings, star-unpacking
//! in calls beyond `*args`.

use anyhow::{bail, Context};

use crate::compiler::ir::*;
use crate::compiler::op::binop_symbol;

// ---------------------------------------------------------------------------
// Lexer (with indentation handling)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum PT {
    NL,
    Indent,
    Dedent,
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    LP,
    RP,
    LB,
    RB,
    LC,
    RC,
    Comma,
    Colon,
    Dot,
    Op(String),
    Kw(String),
}

fn is_kw_word(s: &str) -> bool {
    matches!(
        s,
        "if" | "elif"
            | "else"
            | "while"
            | "for"
            | "def"
            | "return"
            | "import"
            | "from"
            | "as"
            | "break"
            | "continue"
            | "pass"
            | "and"
            | "or"
            | "not"
            | "in"
            | "is"
            | "lambda"
            | "True"
            | "False"
            | "None"
    )
}

fn multi_op(s: &str) -> Option<&'static str> {
    // longest first
    for op in ["//=", "**", "//", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%="] {
        if s.starts_with(op) {
            return Some(op);
        }
    }
    None
}

fn lex(src: &str) -> anyhow::Result<Vec<PT>> {
    let chars: Vec<char> = src.chars().collect();
    let len = chars.len();
    let mut i = 0usize;
    let mut toks = Vec::new();
    let mut indent_stack = vec![0usize];
    let mut depth = 0i32;
    let mut pending_nl = false;

    let measure_indent = |chars: &[char], from: usize| -> usize {
        let mut n = 0;
        let mut j = from;
        while j < chars.len() && (chars[j] == ' ' || chars[j] == '\t') {
            n += if chars[j] == '\t' { 8 } else { 1 };
            j += 1;
        }
        n
    };

    while i < len {
        let c = chars[i];
        if c == '#' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '\n' {
            i += 1;
            if depth == 0 {
                pending_nl = true;
            }
            continue;
        }
        // At the start of a logical line, measure indentation and emit
        // Indent/Dedent before the first real token — whether or not the line
        // begins with whitespace (a line dedented to column 0 still needs a
        // Dedent but has no leading whitespace to trigger the branch below).
        if pending_nl {
            let ind = measure_indent(&chars, i);
            let top = *indent_stack.last().unwrap();
            if ind > top {
                indent_stack.push(ind);
                toks.push(PT::Indent);
            } else if ind < top {
                while *indent_stack.last().unwrap() > ind {
                    indent_stack.pop();
                    toks.push(PT::Dedent);
                    if *indent_stack.last().unwrap() < ind {
                        bail!("inconsistent indentation");
                    }
                }
            }
            pending_nl = false;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // numbers
        if c.is_ascii_digit() || (c == '.' && chars.get(i + 1).map(|x| x.is_ascii_digit()).unwrap_or(false)) {
            let mut s = String::new();
            let is_float = c == '.';
            while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                s.push(chars[i]);
                i += 1;
            }
            if is_float || s.contains('.') {
                let f: f64 = s.parse().context("bad float")?;
                toks.push(PT::Float(f));
            } else {
                let v: i64 = s.parse().context("bad int")?;
                toks.push(PT::Int(v));
            }
            continue;
        }
        // string
        if c == '\'' || c == '"' {
            let quote = c;
            i += 1;
            let mut s = String::new();
            while i < len && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < len {
                    i += 1;
                    match chars[i] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        q if q == quote => s.push(q),
                        o => {
                            s.push('\\');
                            s.push(o);
                        }
                    }
                } else {
                    s.push(chars[i]);
                }
                i += 1;
            }
            if i >= len {
                bail!("unterminated string");
            }
            i += 1; // closing quote
            toks.push(PT::Str(s));
            continue;
        }
        // identifier / keyword
        if c.is_ascii_alphabetic() || c == '_' {
            let mut s = String::new();
            while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                s.push(chars[i]);
                i += 1;
            }
            if is_kw_word(&s) {
                toks.push(PT::Kw(s));
            } else {
                toks.push(PT::Ident(s));
            }
            continue;
        }
        // operators / punctuation
        let rest: String = chars[i..].iter().collect();
        if let Some(op) = multi_op(&rest) {
            toks.push(PT::Op(op.to_string()));
            i += op.len();
            continue;
        }
        match c {
            '(' => {
                depth += 1;
                toks.push(PT::LP);
                i += 1;
            }
            ')' => {
                depth -= 1;
                toks.push(PT::RP);
                i += 1;
            }
            '[' => {
                depth += 1;
                toks.push(PT::LB);
                i += 1;
            }
            ']' => {
                depth -= 1;
                toks.push(PT::RB);
                i += 1;
            }
            '{' => {
                depth += 1;
                toks.push(PT::LC);
                i += 1;
            }
            '}' => {
                depth -= 1;
                toks.push(PT::RC);
                i += 1;
            }
            ',' => {
                toks.push(PT::Comma);
                i += 1;
            }
            ':' => {
                toks.push(PT::Colon);
                i += 1;
            }
            '.' => {
                toks.push(PT::Dot);
                i += 1;
            }
            '+' | '-' | '*' | '/' | '%' | '<' | '>' | '=' => {
                toks.push(PT::Op(c.to_string()));
                i += 1;
            }
            _ => bail!("unexpected character {:?}", c),
        }
    }
    while indent_stack.len() > 1 {
        indent_stack.pop();
        toks.push(PT::Dedent);
    }
    Ok(toks)
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct PyParser {
    toks: Vec<PT>,
    pos: usize,
}

impl PyParser {
    fn peek(&self) -> Option<&PT> {
        self.toks.get(self.pos)
    }
    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }
    fn advance(&mut self) -> Option<PT> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn skip_nl(&mut self) {
        while matches!(self.peek(), Some(PT::NL)) {
            self.pos += 1;
        }
    }
    fn expect(&mut self, want: &PT) -> anyhow::Result<()> {
        match self.peek() {
            Some(t) if t == want => {
                self.pos += 1;
                Ok(())
            }
            other => bail!("expected {:?}, got {:?}", want, other),
        }
    }
    fn expect_ident(&mut self) -> anyhow::Result<String> {
        match self.advance() {
            Some(PT::Ident(s)) => Ok(s),
            other => bail!("expected identifier, got {:?}", other),
        }
    }
    fn is_kw(&self, w: &str) -> bool {
        matches!(self.peek(), Some(PT::Kw(s)) if s == w)
    }
    fn expect_kw(&mut self, w: &str) -> anyhow::Result<()> {
        if self.is_kw(w) {
            self.pos += 1;
            Ok(())
        } else {
            bail!("expected keyword '{}', got {:?}", w, self.peek())
        }
    }

    fn parse_program(&mut self) -> anyhow::Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        self.skip_nl();
        while !self.at_end() {
            if matches!(self.peek(), Some(PT::Dedent)) {
                break;
            }
            stmts.push(self.parse_stmt()?);
            self.skip_nl();
        }
        Ok(stmts)
    }

    fn parse_block(&mut self) -> anyhow::Result<Vec<Stmt>> {
        self.skip_nl();
        self.expect(&PT::Indent)?;
        let mut stmts = Vec::new();
        loop {
            if matches!(self.peek(), Some(PT::Dedent)) {
                self.advance();
                break;
            }
            if self.at_end() {
                break;
            }
            stmts.push(self.parse_stmt()?);
            self.skip_nl();
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> anyhow::Result<Stmt> {
        self.skip_nl();
        if let Some(PT::Kw(w)) = self.peek() {
            match w.as_str() {
                "def" => return self.parse_def(),
                "if" => return self.parse_if(),
                "while" => return self.parse_while(),
                "for" => return self.parse_for(),
                "return" => return self.parse_return(),
                "import" => return self.parse_import(),
                "from" => return self.parse_from(),
                "break" => {
                    self.advance();
                    self.skip_nl();
                    return Ok(Stmt::Break);
                }
                "continue" => {
                    self.advance();
                    self.skip_nl();
                    return Ok(Stmt::Continue);
                }
                "pass" => {
                    self.advance();
                    self.skip_nl();
                    return Ok(Stmt::Pass);
                }
                _ => {}
            }
        }
        self.parse_expr_stmt()
    }

    fn parse_expr_stmt(&mut self) -> anyhow::Result<Stmt> {
        let e = self.parse_expr()?;
        match self.peek() {
            Some(PT::Op(o)) if o == "=" => {
                self.advance();
                let v = self.parse_expr()?;
                self.skip_nl();
                Ok(Stmt::Assign { target: e, value: v })
            }
            Some(PT::Op(o)) if is_aug(o) => {
                let op = aug_op(o).context("bad aug op")?;
                self.advance();
                let v = self.parse_expr()?;
                self.skip_nl();
                Ok(Stmt::Aug { op, target: e, value: v })
            }
            _ => {
                self.skip_nl();
                Ok(Stmt::Expr(e))
            }
        }
    }

    fn parse_def(&mut self) -> anyhow::Result<Stmt> {
        self.expect_kw("def")?;
        let name = self.expect_ident()?;
        self.expect(&PT::LP)?;
        let mut params = Vec::new();
        while !matches!(self.peek(), Some(PT::RP)) {
            let kind = if matches!(self.peek(), Some(PT::Op(o)) if o == "*") {
                self.advance();
                if matches!(self.peek(), Some(PT::Op(o)) if o == "*") {
                    self.advance();
                    ParamKind::DoubleStar
                } else {
                    ParamKind::Star
                }
            } else {
                ParamKind::Normal
            };
            let pname = self.expect_ident()?;
            params.push(Param { name: pname, kind });
            if matches!(self.peek(), Some(PT::Comma)) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&PT::RP)?;
        self.expect(&PT::Colon)?;
        let body = self.parse_block()?;
        Ok(Stmt::Def { name, params, body })
    }

    fn parse_if(&mut self) -> anyhow::Result<Stmt> {
        self.expect_kw("if")?;
        let test = self.parse_expr()?;
        self.expect(&PT::Colon)?;
        let body = self.parse_block()?;
        let mut branches = vec![(Some(test), body)];
        loop {
            if self.is_kw("elif") {
                self.advance();
                let t = self.parse_expr()?;
                self.expect(&PT::Colon)?;
                let b = self.parse_block()?;
                branches.push((Some(t), b));
            } else if self.is_kw("else") {
                self.advance();
                self.expect(&PT::Colon)?;
                let b = self.parse_block()?;
                branches.push((None, b));
                break;
            } else {
                break;
            }
        }
        Ok(Stmt::If { branches })
    }

    fn parse_while(&mut self) -> anyhow::Result<Stmt> {
        self.expect_kw("while")?;
        let test = self.parse_expr()?;
        self.expect(&PT::Colon)?;
        let body = self.parse_block()?;
        Ok(Stmt::While { test, body })
    }

    fn parse_for(&mut self) -> anyhow::Result<Stmt> {
        self.expect_kw("for")?;
        let target = self.parse_expr()?;
        self.expect_kw("in")?;
        let iter = self.parse_expr()?;
        self.expect(&PT::Colon)?;
        let body = self.parse_block()?;
        Ok(Stmt::For { target, iter, body })
    }

    fn parse_return(&mut self) -> anyhow::Result<Stmt> {
        self.expect_kw("return")?;
        if matches!(self.peek(), Some(PT::NL)) || self.at_end() {
            self.skip_nl();
            Ok(Stmt::Ret(None))
        } else {
            let v = self.parse_expr()?;
            self.skip_nl();
            Ok(Stmt::Ret(Some(v)))
        }
    }

    fn parse_import(&mut self) -> anyhow::Result<Stmt> {
        self.expect_kw("import")?;
        let mut names = Vec::new();
        loop {
            let n = self.expect_ident()?;
            let alias = if self.is_kw("as") {
                self.advance();
                Some(self.expect_ident()?)
            } else {
                None
            };
            names.push((n, alias));
            if matches!(self.peek(), Some(PT::Comma)) {
                self.advance();
            } else {
                break;
            }
        }
        self.skip_nl();
        Ok(Stmt::Import { names })
    }

    fn parse_from(&mut self) -> anyhow::Result<Stmt> {
        self.expect_kw("from")?;
        let module = self.expect_ident()?;
        self.expect_kw("import")?;
        let mut names = Vec::new();
        loop {
            let n = self.expect_ident()?;
            let alias = if self.is_kw("as") {
                self.advance();
                Some(self.expect_ident()?)
            } else {
                None
            };
            names.push((n, alias));
            if matches!(self.peek(), Some(PT::Comma)) {
                self.advance();
            } else {
                break;
            }
        }
        self.skip_nl();
        Ok(Stmt::FromImport { module, names })
    }

    // -- expressions (precedence climbing) --

    fn parse_expr(&mut self) -> anyhow::Result<Expr> {
        self.parse_or()
    }
    fn parse_or(&mut self) -> anyhow::Result<Expr> {
        let mut left = self.parse_and()?;
        while self.is_kw("or") {
            self.advance();
            let r = self.parse_and()?;
            left = Expr::Bin { op: BinOp::Or, l: Box::new(left), r: Box::new(r) };
        }
        Ok(left)
    }
    fn parse_and(&mut self) -> anyhow::Result<Expr> {
        let mut left = self.parse_not()?;
        while self.is_kw("and") {
            self.advance();
            let r = self.parse_not()?;
            left = Expr::Bin { op: BinOp::And, l: Box::new(left), r: Box::new(r) };
        }
        Ok(left)
    }
    fn parse_not(&mut self) -> anyhow::Result<Expr> {
        if self.is_kw("not") {
            self.advance();
            let e = self.parse_not()?;
            return Ok(Expr::Un { op: UnOp::Not, e: Box::new(e) });
        }
        self.parse_cmp()
    }
    fn parse_cmp(&mut self) -> anyhow::Result<Expr> {
        let mut left = self.parse_add()?;
        loop {
            let op = match self.peek() {
                Some(PT::Op(o)) if o == "==" => BinOp::Eq,
                Some(PT::Op(o)) if o == "!=" => BinOp::Ne,
                Some(PT::Op(o)) if o == "<" => BinOp::Lt,
                Some(PT::Op(o)) if o == ">" => BinOp::Gt,
                Some(PT::Op(o)) if o == "<=" => BinOp::Le,
                Some(PT::Op(o)) if o == ">=" => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let r = self.parse_add()?;
            left = Expr::Bin { op, l: Box::new(left), r: Box::new(r) };
        }
        Ok(left)
    }
    fn parse_add(&mut self) -> anyhow::Result<Expr> {
        let mut left = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Some(PT::Op(o)) if o == "+" => BinOp::Add,
                Some(PT::Op(o)) if o == "-" => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let r = self.parse_mul()?;
            left = Expr::Bin { op, l: Box::new(left), r: Box::new(r) };
        }
        Ok(left)
    }
    fn parse_mul(&mut self) -> anyhow::Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Some(PT::Op(o)) if o == "*" => BinOp::Mul,
                Some(PT::Op(o)) if o == "/" => BinOp::Div,
                Some(PT::Op(o)) if o == "//" => BinOp::FloorDiv,
                Some(PT::Op(o)) if o == "%" => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let r = self.parse_unary()?;
            left = Expr::Bin { op, l: Box::new(left), r: Box::new(r) };
        }
        Ok(left)
    }
    fn parse_unary(&mut self) -> anyhow::Result<Expr> {
        if matches!(self.peek(), Some(PT::Op(o)) if o == "-") {
            self.advance();
            let e = self.parse_unary()?;
            return Ok(Expr::Un { op: UnOp::Neg, e: Box::new(e) });
        }
        if matches!(self.peek(), Some(PT::Op(o)) if o == "+") {
            self.advance();
            return self.parse_unary();
        }
        self.parse_power()
    }
    fn parse_power(&mut self) -> anyhow::Result<Expr> {
        let base = self.parse_postfix()?;
        if matches!(self.peek(), Some(PT::Op(o)) if o == "**") {
            self.advance();
            let exp = self.parse_unary()?;
            return Ok(Expr::Bin { op: BinOp::Pow, l: Box::new(base), r: Box::new(exp) });
        }
        Ok(base)
    }
    fn parse_postfix(&mut self) -> anyhow::Result<Expr> {
        let mut e = self.parse_atom()?;
        loop {
            match self.peek() {
                Some(PT::LP) => {
                    self.advance();
                    let args = self.parse_call_args()?;
                    e = Expr::Call { func: Box::new(e), args };
                }
                Some(PT::Dot) => {
                    self.advance();
                    let n = self.expect_ident()?;
                    e = Expr::Attr { obj: Box::new(e), name: n };
                }
                Some(PT::LB) => {
                    self.advance();
                    let idx = self.parse_expr()?;
                    self.expect(&PT::RB)?;
                    e = Expr::Sub { obj: Box::new(e), idx: Box::new(idx) };
                }
                _ => break,
            }
        }
        Ok(e)
    }
    fn parse_call_args(&mut self) -> anyhow::Result<Vec<Arg>> {
        let mut args = Vec::new();
        while !matches!(self.peek(), Some(PT::RP)) {
            if let Some(PT::Ident(n)) = self.peek().cloned() {
                // lookahead for '='
                if matches!(self.toks.get(self.pos + 1), Some(PT::Op(o)) if o == "=") {
                    self.advance();
                    self.advance();
                    let v = self.parse_expr()?;
                    args.push(Arg { name: Some(n), value: v });
                    if matches!(self.peek(), Some(PT::Comma)) {
                        self.advance();
                    }
                    continue;
                }
            }
            let v = self.parse_expr()?;
            args.push(Arg { name: None, value: v });
            if matches!(self.peek(), Some(PT::Comma)) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(&PT::RP)?;
        Ok(args)
    }
    fn parse_atom(&mut self) -> anyhow::Result<Expr> {
        match self.peek().cloned() {
            Some(PT::Int(i)) => {
                self.advance();
                Ok(Expr::Lit(Lit::Int(i)))
            }
            Some(PT::Float(f)) => {
                self.advance();
                Ok(Expr::Lit(Lit::Float(f)))
            }
            Some(PT::Str(s)) => {
                self.advance();
                Ok(Expr::Lit(Lit::Str(s)))
            }
            Some(PT::Kw(w)) if w == "True" => {
                self.advance();
                Ok(Expr::Lit(Lit::Bool(true)))
            }
            Some(PT::Kw(w)) if w == "False" => {
                self.advance();
                Ok(Expr::Lit(Lit::Bool(false)))
            }
            Some(PT::Kw(w)) if w == "None" => {
                self.advance();
                Ok(Expr::Lit(Lit::None))
            }
            Some(PT::Ident(n)) => {
                self.advance();
                Ok(Expr::Name(n))
            }
            Some(PT::LP) => {
                self.advance();
                if matches!(self.peek(), Some(PT::RP)) {
                    self.advance();
                    return Ok(Expr::Tuple(vec![]));
                }
                let first = self.parse_expr()?;
                if matches!(self.peek(), Some(PT::Comma)) {
                    self.advance();
                    let mut items = vec![first];
                    while !matches!(self.peek(), Some(PT::RP)) {
                        if matches!(self.peek(), Some(PT::Comma)) {
                            self.advance();
                            continue;
                        }
                        items.push(self.parse_expr()?);
                    }
                    self.expect(&PT::RP)?;
                    Ok(Expr::Tuple(items))
                } else {
                    self.expect(&PT::RP)?;
                    Ok(first)
                }
            }
            Some(PT::LB) => {
                self.advance();
                let mut items = Vec::new();
                while !matches!(self.peek(), Some(PT::RB)) {
                    if matches!(self.peek(), Some(PT::Comma)) {
                        self.advance();
                        continue;
                    }
                    items.push(self.parse_expr()?);
                }
                self.expect(&PT::RB)?;
                Ok(Expr::List(items))
            }
            Some(PT::LC) => {
                self.advance();
                let mut items = Vec::new();
                while !matches!(self.peek(), Some(PT::RC)) {
                    if matches!(self.peek(), Some(PT::Comma)) {
                        self.advance();
                        continue;
                    }
                    let k = self.parse_expr()?;
                    self.expect(&PT::Colon)?;
                    let v = self.parse_expr()?;
                    items.push((k, v));
                }
                self.expect(&PT::RC)?;
                Ok(Expr::Dict(items))
            }
            other => bail!("unexpected token in expression: {:?}", other),
        }
    }
}

fn is_aug(o: &str) -> bool {
    matches!(o, "+=" | "-=" | "*=" | "/=" | "//=" | "%=" | "**=")
}
fn aug_op(o: &str) -> anyhow::Result<BinOp> {
    Ok(match o {
        "+=" => BinOp::Add,
        "-=" => BinOp::Sub,
        "*=" => BinOp::Mul,
        "/=" => BinOp::Div,
        "//=" => BinOp::FloorDiv,
        "%=" => BinOp::Mod,
        "**=" => BinOp::Pow,
        _ => anyhow::bail!("bad aug op"),
    })
}

/// Parse a Python-subset source string into IR.
pub fn parse_python(src: &str) -> anyhow::Result<Vec<Stmt>> {
    let toks = lex(src)?;
    let mut p = PyParser { toks, pos: 0 };
    p.parse_program()
}

// ---------------------------------------------------------------------------
// Emitter (IR -> Python)
// ---------------------------------------------------------------------------

fn py_lit(l: &Lit) -> String {
    match l {
        Lit::Int(i) => i.to_string(),
        Lit::Float(f) => {
            let s = f.to_string();
            if s.contains('.') || s.contains('e') || s.contains('E') {
                s
            } else {
                format!("{}.0", s)
            }
        }
        Lit::Str(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")),
        Lit::Bool(b) => if *b { "True".into() } else { "False".into() },
        Lit::None => "None".into(),
    }
}

fn py_expr(e: &Expr) -> String {
    match e {
        Expr::Lit(l) => py_lit(l),
        Expr::Name(n) => n.clone(),
        Expr::Bin { op, l, r } => format!("({} {} {})", py_expr(l), binop_symbol(*op), py_expr(r)),
        Expr::Un { op: UnOp::Not, e } => format!("(not {})", py_expr(e)),
        Expr::Un { op: UnOp::Neg, e } => format!("(-{})", py_expr(e)),
        Expr::Call { func, args } => {
            let a: Vec<String> = args
                .iter()
                .map(|arg| match &arg.name {
                    Some(n) => format!("{}={}", n, py_expr(&arg.value)),
                    None => py_expr(&arg.value),
                })
                .collect();
            format!("{}(", py_expr(func)) + &a.join(", ") + ")"
        }
        Expr::Attr { obj, name } => format!("{}.{}", py_expr(obj), name),
        Expr::Sub { obj, idx } => format!("{}[{}]", py_expr(obj), py_expr(idx)),
        Expr::List(items) => {
            let inner: Vec<String> = items.iter().map(py_expr).collect();
            format!("[{}]", inner.join(", "))
        }
        Expr::Tuple(items) => {
            let inner: Vec<String> = items.iter().map(py_expr).collect();
            if items.len() == 1 {
                format!("({},)", inner[0])
            } else {
                format!("({})", inner.join(", "))
            }
        }
        Expr::Dict(items) => {
            let inner: Vec<String> = items
                .iter()
                .map(|(k, v)| format!("{}: {}", py_expr(k), py_expr(v)))
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
        Expr::Ternary { test, then, els } => {
            format!("{} if {} else {}", py_expr(then), py_expr(test), py_expr(els))
        }
    }
}

fn py_param(p: &Param) -> String {
    match p.kind {
        ParamKind::Normal => p.name.clone(),
        ParamKind::Star => format!("*{}", p.name),
        ParamKind::DoubleStar => format!("**{}", p.name),
    }
}

fn py_block(stmts: &[Stmt], ind: usize) -> String {
    let pad = "    ".repeat(ind + 1);
    let close = "    ".repeat(ind);
    let lines: Vec<String> = stmts.iter().map(|s| format!("{}{}", pad, py_stmt(s, ind + 1))).collect();
    format!("{}\n{}", lines.join("\n"), close)
}

fn py_stmt(s: &Stmt, ind: usize) -> String {
    match s {
        Stmt::Expr(e) => py_expr(e),
        Stmt::Break => "break".into(),
        Stmt::Continue => "continue".into(),
        Stmt::Pass => "pass".into(),
        Stmt::Assign { target, value } => format!("{} = {}", py_expr(target), py_expr(value)),
        Stmt::Aug { op, target, value } => {
            format!("{} {}= {}", py_expr(target), binop_symbol(*op), py_expr(value))
        }
        Stmt::Def { name, params, body } => {
            let ps: Vec<String> = params.iter().map(py_param).collect();
            format!("def {}({}):\n{}", name, ps.join(", "), py_block(body, ind))
        }
        Stmt::Ret(None) => "return".into(),
        Stmt::Ret(Some(e)) => format!("return {}", py_expr(e)),
        Stmt::If { branches } => {
            let mut out = String::new();
            for (i, (test, body)) in branches.iter().enumerate() {
                let kw = match test {
                    Some(_) if i == 0 => "if",
                    Some(_) => "elif",
                    None => "else",
                };
                match test {
                    Some(t) => out.push_str(&format!("{} {}:\n{}\n", kw, py_expr(t), py_block(body, ind))),
                    None => out.push_str(&format!("{}:\n{}\n", kw, py_block(body, ind))),
                }
            }
            out.trim_end().to_string()
        }
        Stmt::While { test, body } => {
            format!("while {}:\n{}\n{}", py_expr(test), py_block(body, ind), "    ".repeat(ind))
                .trim_end()
                .to_string()
        }
        Stmt::For { target, iter, body } => format!(
            "for {} in {}:\n{}\n{}",
            py_expr(target),
            py_expr(iter),
            py_block(body, ind),
            "    ".repeat(ind)
        )
        .trim_end()
        .to_string(),
        Stmt::Import { names } => {
            let ns: Vec<String> = names
                .iter()
                .map(|(n, a)| match a {
                    Some(a) => format!("{} as {}", n, a),
                    None => n.clone(),
                })
                .collect();
            format!("import {}", ns.join(", "))
        }
        Stmt::FromImport { module, names } => {
            let ns: Vec<String> = names
                .iter()
                .map(|(n, a)| match a {
                    Some(a) => format!("{} as {}", n, a),
                    None => n.clone(),
                })
                .collect();
            format!("from {} import {}", module, ns.join(", "))
        }
    }
}

/// Emit IR as Python source.
pub fn emit_python(stmts: &[Stmt]) -> String {
    let lines: Vec<String> = stmts.iter().map(|s| py_stmt(s, 0)).collect();
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::glyph;
    use crate::seed::permute::build;
    use crate::seed::prng::from_entropy;

    fn rt(src: &str) {
        let stmts = parse_python(src).expect("parse");
        let out = emit_python(&stmts);
        let stmts2 = parse_python(&out).expect("re-parse");
        assert_eq!(stmts, stmts2, "Python round-trip changed IR:\n---\n{}\n---", out);
    }

    #[test]
    fn rt_def_return() {
        rt("def add(a, b):\n    return a + b\n");
    }

    #[test]
    fn rt_control_flow() {
        rt(
            "x = 1\nif x > 0:\n    print(\"pos\")\nelif x < 0:\n    print(\"neg\")\nelse:\n    pass\nwhile x > 0:\n    x = x - 1\nfor i in range(10):\n    print(i)\n",
        );
    }

    #[test]
    fn rt_expressions() {
        rt("y = (a * b) + -c == d and not e or f[0]\nz = foo(1, 2, k=3).bar[0]\nd = {1: 2, 3: 4}\nt = (1, 2, 3)\n");
    }

    #[test]
    fn rt_aug_and_import() {
        rt(
            "count += 1\ntotal *= 2\nimport os, sys as system\nfrom math import sqrt, pi as PI\n",
        );
    }

    #[test]
    fn py_to_glyph_to_py() {
        let src = "def add(a, b):\n    return a + b\n";
        let stmts = parse_python(src).unwrap();
        let map = build(&from_entropy());
        let g = glyph::emit_glyph(&stmts, &map);
        let back = glyph::parse_glyph(&g, &map).unwrap();
        assert_eq!(stmts, back, "glyph round-trip mismatch:\n{}", g);
        let py2 = emit_python(&back);
        assert_eq!(stmts, parse_python(&py2).unwrap());
    }
}

