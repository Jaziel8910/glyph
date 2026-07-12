//! Glyph S-expression source: lexer, parser (glyph text -> IR) and emitter (IR
//! -> glyph text). A `.glf` body is an S-expression where *every operator* is a
//! single dialect glyph char (mapped from a canonical op name) and data
//! (names, numbers, strings) is written literally. The dialect map comes from
//! the seed, so the same source is unreadable without it.

use std::collections::HashSet;

use anyhow::{bail, Context};

use crate::compiler::ir::*;
use crate::compiler::op::*;
use crate::seed::permute::DialectMap;

#[derive(Debug, Clone, PartialEq)]
enum Tk {
    LP,
    RP,
    Glyph(char),
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    NoneTok,
}

struct Lexer<'a> {
    chars: std::str::Chars<'a>,
    glyphs: &'a HashSet<char>,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a str, glyphs: &'a HashSet<char>) -> Self {
        Lexer { chars: text.chars(), glyphs }
    }

    fn next_tok(&mut self) -> anyhow::Result<Option<Tk>> {
        while let Some(c) = self.chars.next() {
            if c.is_whitespace() {
                continue;
            }
            if c == '(' {
                return Ok(Some(Tk::LP));
            }
            if c == ')' {
                return Ok(Some(Tk::RP));
            }
            if self.glyphs.contains(&c) {
                return Ok(Some(Tk::Glyph(c)));
            }
            if c == '"' {
                let mut s = String::new();
                loop {
                    match self.chars.next() {
                        None => bail!("unterminated string"),
                        Some('"') => break,
                        Some('\\') => match self.chars.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some('r') => s.push('\r'),
                            Some('\\') => s.push('\\'),
                            Some('"') => s.push('"'),
                            Some(o) => {
                                s.push('\\');
                                s.push(o);
                            }
                            None => bail!("bad escape in string"),
                        },
                        Some(o) => s.push(o),
                    }
                }
                return Ok(Some(Tk::Str(s)));
            }
            if c.is_ascii_digit() || (c == '-' && self.chars.clone().next().map(|n| n.is_ascii_digit()).unwrap_or(false)) {
                let mut s = String::new();
                s.push(c);
                while let Some(n) = self.chars.clone().next() {
                    if n.is_ascii_digit() || n == '.' {
                        s.push(n);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                if s.contains('.') {
                    let f: f64 = s.parse().context("bad float")?;
                    return Ok(Some(Tk::Float(f)));
                }
                let i: i64 = s.parse().context("bad int")?;
                return Ok(Some(Tk::Int(i)));
            }
            if c.is_ascii_alphabetic() || c == '_' {
                let mut s = String::new();
                s.push(c);
                while let Some(n) = self.chars.clone().next() {
                    if n.is_ascii_alphanumeric() || n == '_' {
                        s.push(n);
                        self.chars.next();
                    } else {
                        break;
                    }
                }
                return Ok(Some(match s.as_str() {
                    "True" => Tk::Bool(true),
                    "False" => Tk::Bool(false),
                    "None" => Tk::NoneTok,
                    _ => Tk::Ident(s),
                }));
            }
            bail!("unexpected character: {:?}", c);
        }
        Ok(None)
    }
}

fn lex(text: &str, glyphs: &HashSet<char>) -> anyhow::Result<Vec<Tk>> {
    let mut lx = Lexer::new(text, glyphs);
    let mut out = Vec::new();
    while let Some(t) = lx.next_tok()? {
        out.push(t);
    }
    Ok(out)
}

struct Parser<'m> {
    toks: Vec<Tk>,
    pos: usize,
    map: &'m DialectMap,
}

impl<'m> Parser<'m> {
    fn peek(&self) -> Option<&Tk> {
        self.toks.get(self.pos)
    }
    fn next(&mut self) -> Option<Tk> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn expect_lp(&mut self) -> anyhow::Result<()> {
        match self.next() {
            Some(Tk::LP) => Ok(()),
            other => bail!("expected '(' , got {:?}", other),
        }
    }
    fn expect_rp(&mut self) -> anyhow::Result<()> {
        match self.next() {
            Some(Tk::RP) => Ok(()),
            other => bail!("expected ')' , got {:?}", other),
        }
    }
    fn expect_ident(&mut self) -> anyhow::Result<String> {
        match self.next() {
            Some(Tk::Ident(s)) => Ok(s),
            other => bail!("expected identifier, got {:?}", other),
        }
    }
    fn expect_glyph(&mut self) -> anyhow::Result<&'m str> {
        match self.next() {
            Some(Tk::Glyph(g)) => match self.map.glyph_to_op.get(&g) {
                Some(s) => Ok(s.as_str()),
                None => bail!("unknown glyph operator"),
            },
            other => bail!("expected glyph operator, got {:?}", other),
        }
    }

    fn parse_program(&mut self) -> anyhow::Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while self.peek().is_some() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> anyhow::Result<Stmt> {
        match self.peek() {
            Some(Tk::Glyph(g)) => {
                let name = self.map.glyph_to_op.get(g).map(|s| s.as_str()).unwrap().to_string();
                if is_nullary_stmt(&name) {
                    self.next();
                    return Ok(match name.as_str() {
                        "break" => Stmt::Break,
                        "continue" => Stmt::Continue,
                        _ => Stmt::Pass,
                    });
                }
                bail!("unexpected standalone glyph '{}'", name)
            }
            Some(Tk::LP) => {
                self.next();
                let name = self.expect_glyph()?.to_string();
                if is_stmt_form(&name) {
                    self.parse_stmt_form(&name)
                } else if is_expr_form(&name) {
                    let e = self.parse_expr_form(&name)?;
                    Ok(Stmt::Expr(e))
                } else {
                    bail!("unknown statement operator '{}'", name)
                }
            }
            Some(Tk::Ident(_))
            | Some(Tk::Int(_))
            | Some(Tk::Float(_))
            | Some(Tk::Str(_))
            | Some(Tk::Bool(_))
            | Some(Tk::NoneTok) => Ok(Stmt::Expr(self.parse_expr()?)),
            other => bail!("expected statement, got {:?}", other),
        }
    }

    fn parse_block(&mut self) -> anyhow::Result<Vec<Stmt>> {
        self.expect_lp()?;
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Some(Tk::RP) | None) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect_rp()?;
        Ok(stmts)
    }

    fn parse_stmt_form(&mut self, name: &str) -> anyhow::Result<Stmt> {
        let stmt = match name {
            "def" => {
                let fname = self.expect_ident()?;
                self.expect_lp()?;
                let mut params = Vec::new();
                while !matches!(self.peek(), Some(Tk::RP)) {
                    match self.peek() {
                        Some(Tk::Ident(p)) => {
                            let p = p.clone();
                            self.next();
                            params.push(Param { name: p, kind: ParamKind::Normal });
                        }
                        Some(Tk::LP) => {
                            self.next();
                            let pk = self.expect_glyph()?;
                            let p = self.expect_ident()?;
                            self.expect_rp()?;
                            let kind = match pk {
                                "star" => ParamKind::Star,
                                "dstar" => ParamKind::DoubleStar,
                                other => bail!("bad param kind '{}'", other),
                            };
                            params.push(Param { name: p, kind });
                        }
                        other => bail!("bad param, got {:?}", other),
                    }
                }
                self.expect_rp()?;
                let body = self.parse_block()?;
                self.expect_rp()?;
                Stmt::Def { name: fname, params, body }
            }
            "ret" => {
                let v = if matches!(self.peek(), Some(Tk::RP)) {
                    None
                } else {
                    Some(self.parse_expr()?)
                };
                self.expect_rp()?;
                Stmt::Ret(v)
            }
            "if" => {
                let mut branches = Vec::new();
                while !matches!(self.peek(), Some(Tk::RP)) {
                    self.expect_lp()?;
                    // clause head: either `else` ident or a test expression
                    match self.peek() {
                        Some(Tk::Ident(w)) if w == "else" => {
                            self.next();
                            let body = self.parse_block()?;
                            self.expect_rp()?;
                            branches.push((None, body));
                        }
                        _ => {
                            let test = self.parse_expr()?;
                            let body = self.parse_block()?;
                            self.expect_rp()?;
                            branches.push((Some(test), body));
                        }
                    }
                }
                self.expect_rp()?;
                Stmt::If { branches }
            }
            "while" => {
                let test = self.parse_expr()?;
                let body = self.parse_block()?;
                self.expect_rp()?;
                Stmt::While { test, body }
            }
            "for" => {
                let target = self.parse_expr()?;
                let iter = self.parse_expr()?;
                let body = self.parse_block()?;
                self.expect_rp()?;
                Stmt::For { target, iter, body }
            }
            "assign" => {
                let target = self.parse_expr()?;
                let value = self.parse_expr()?;
                self.expect_rp()?;
                Stmt::Assign { target, value }
            }
            "aug" => {
                let opname = self.expect_glyph()?.to_string();
                let op = binop_from_name(&opname).ok_or_else(|| anyhow::anyhow!("bad aug op"))?;
                let target = self.parse_expr()?;
                let value = self.parse_expr()?;
                self.expect_rp()?;
                Stmt::Aug { op, target, value }
            }
            "import" => {
                let mut names = Vec::new();
                while !matches!(self.peek(), Some(Tk::RP)) {
                    match self.peek() {
                        Some(Tk::Ident(n)) => {
                            let n = n.clone();
                            self.next();
                            names.push((n, None));
                        }
                        Some(Tk::LP) => {
                            self.next();
                            let pk = self.expect_glyph()?;
                            if pk != "as_alias" {
                                bail!("expected as_alias in import");
                            }
                            let n = self.expect_ident()?;
                            let a = self.expect_ident()?;
                            self.expect_rp()?;
                            names.push((n, Some(a)));
                        }
                        other => bail!("bad import item {:?}", other),
                    }
                }
                self.expect_rp()?;
                Stmt::Import { names }
            }
            "from_import" => {
                let module = self.expect_ident()?;
                let sep = self.expect_glyph()?.to_string();
                if sep != "import" {
                    bail!("expected import separator in from_import");
                }
                let mut names = Vec::new();
                while !matches!(self.peek(), Some(Tk::RP)) {
                    match self.peek() {
                        Some(Tk::Ident(n)) => {
                            let n = n.clone();
                            self.next();
                            names.push((n, None));
                        }
                        Some(Tk::LP) => {
                            self.next();
                            let pk = self.expect_glyph()?;
                            if pk != "as_alias" {
                                bail!("expected as_alias in from_import");
                            }
                            let n = self.expect_ident()?;
                            let a = self.expect_ident()?;
                            self.expect_rp()?;
                            names.push((n, Some(a)));
                        }
                        other => bail!("bad from item {:?}", other),
                    }
                }
                self.expect_rp()?;
                Stmt::FromImport { module, names }
            }
            other => bail!("unknown statement operator '{}'", other),
        };
        Ok(stmt)
    }

    fn parse_expr(&mut self) -> anyhow::Result<Expr> {
        match self.peek() {
            Some(Tk::LP) => {
                self.next();
                let name = self.expect_glyph()?.to_string();
                if is_expr_form(&name) {
                    self.parse_expr_form(&name)
                } else {
                    bail!("expected expression operator, got '{}'", name)
                }
            }
            Some(Tk::Ident(s)) => {
                let s = s.clone();
                self.next();
                Ok(Expr::Name(s))
            }
            Some(Tk::Int(i)) => {
                let i = *i;
                self.next();
                Ok(Expr::Lit(Lit::Int(i)))
            }
            Some(Tk::Float(f)) => {
                let f = *f;
                self.next();
                Ok(Expr::Lit(Lit::Float(f)))
            }
            Some(Tk::Str(s)) => {
                let s = s.clone();
                self.next();
                Ok(Expr::Lit(Lit::Str(s)))
            }
            Some(Tk::Bool(b)) => {
                let b = *b;
                self.next();
                Ok(Expr::Lit(Lit::Bool(b)))
            }
            Some(Tk::NoneTok) => {
                self.next();
                Ok(Expr::Lit(Lit::None))
            }
            other => bail!("expected expression, got {:?}", other),
        }
    }

    fn parse_expr_form(&mut self, name: &str) -> anyhow::Result<Expr> {
        let e = match name {
            "not" => {
                let e = self.parse_expr()?;
                self.expect_rp()?;
                Expr::Un { op: UnOp::Not, e: Box::new(e) }
            }
            "neg" => {
                let e = self.parse_expr()?;
                self.expect_rp()?;
                Expr::Un { op: UnOp::Neg, e: Box::new(e) }
            }
            "call" => {
                let func = self.parse_expr()?;
                let mut args = Vec::new();
                while !matches!(self.peek(), Some(Tk::RP)) {
                    if matches!(self.peek(), Some(Tk::LP)) {
                        // lookahead: kw arg?
                        let save = self.pos;
                        self.next();
                        let head = self.expect_glyph()?.to_string();
                        if head == "as_alias" {
                            let n = self.expect_ident()?;
                            let v = self.parse_expr()?;
                            self.expect_rp()?;
                            args.push(Arg { name: Some(n), value: v });
                            continue;
                        } else {
                            self.pos = save; // rewind, parse as normal expr
                        }
                    }
                    args.push(Arg { name: None, value: self.parse_expr()? });
                }
                self.expect_rp()?;
                Expr::Call { func: Box::new(func), args }
            }
            "attr" => {
                let obj = self.parse_expr()?;
                let name = self.expect_ident()?;
                self.expect_rp()?;
                Expr::Attr { obj: Box::new(obj), name }
            }
            "subscript" => {
                let obj = self.parse_expr()?;
                let idx = self.parse_expr()?;
                self.expect_rp()?;
                Expr::Sub { obj: Box::new(obj), idx: Box::new(idx) }
            }
            "list_of" => {
                let mut items = Vec::new();
                while !matches!(self.peek(), Some(Tk::RP)) {
                    items.push(self.parse_expr()?);
                }
                self.expect_rp()?;
                Expr::List(items)
            }
            "tuple" => {
                let mut items = Vec::new();
                while !matches!(self.peek(), Some(Tk::RP)) {
                    items.push(self.parse_expr()?);
                }
                self.expect_rp()?;
                Expr::Tuple(items)
            }
            "dict_of" => {
                let mut items = Vec::new();
                while !matches!(self.peek(), Some(Tk::RP)) {
                    self.expect_lp()?;
                    let k = self.parse_expr()?;
                    let v = self.parse_expr()?;
                    self.expect_rp()?;
                    items.push((k, v));
                }
                self.expect_rp()?;
                Expr::Dict(items)
            }
            "ternary" => {
                let test = self.parse_expr()?;
                let then = self.parse_expr()?;
                let els = self.parse_expr()?;
                self.expect_rp()?;
                Expr::Ternary {
                    test: Box::new(test),
                    then: Box::new(then),
                    els: Box::new(els),
                }
            }
            other => {
                if let Some(op) = binop_from_name(other) {
                    let l = self.parse_expr()?;
                    let r = self.parse_expr()?;
                    self.expect_rp()?;
                    Expr::Bin { op, l: Box::new(l), r: Box::new(r) }
                } else {
                    bail!("unknown expression operator '{}'", other)
                }
            }
        };
        Ok(e)
    }
}

/// Parse glyph S-expression text (the body after the header) into IR.
pub fn parse_glyph(text: &str, map: &DialectMap) -> anyhow::Result<Vec<Stmt>> {
    let glyphs: HashSet<char> = map.glyph_to_op.keys().copied().collect();
    let toks = lex(text, &glyphs)?;
    let mut p = Parser { toks, pos: 0, map };
    p.parse_program()
}

fn g(map: &DialectMap, name: &str) -> char {
    *map.op_to_glyph
        .get(name)
        .unwrap_or_else(|| panic!("op '{}' missing from dialect", name))
}

fn emit_block(stmts: &[Stmt], map: &DialectMap, ind: usize) -> String {
    if stmts.is_empty() {
        return "()".to_string();
    }
    let pad = "  ".repeat(ind + 1);
    let close = "  ".repeat(ind);
    let lines: Vec<String> = stmts.iter().map(|s| format!("{}{}", pad, emit_stmt(s, map, ind + 1))).collect();
    format!("(\n{}\n{})", lines.join("\n"), close)
}

fn emit_expr(e: &Expr, map: &DialectMap) -> String {
    match e {
        Expr::Lit(Lit::Int(i)) => i.to_string(),
        Expr::Lit(Lit::Float(f)) => f.to_string(),
        Expr::Lit(Lit::Str(s)) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")),
        Expr::Lit(Lit::Bool(b)) => if *b { "True".into() } else { "False".into() },
        Expr::Lit(Lit::None) => "None".into(),
        Expr::Name(n) => n.clone(),
        Expr::Bin { op, l, r } => format!("({} {} {})", g(map, match op { BinOp::Add=>"add",BinOp::Sub=>"sub",BinOp::Mul=>"mul",BinOp::Div=>"div",BinOp::FloorDiv=>"floordiv",BinOp::Mod=>"mod",BinOp::Pow=>"pow",BinOp::Eq=>"eq",BinOp::Ne=>"ne",BinOp::Lt=>"lt",BinOp::Gt=>"gt",BinOp::Le=>"le",BinOp::Ge=>"ge",BinOp::And=>"and",BinOp::Or=>"or" }), emit_expr(l,map), emit_expr(r,map)),
        Expr::Un { op, e: inner } => {
            let tag = match op { UnOp::Not => "not", UnOp::Neg => "neg" };
            format!("({} {})", g(map, tag), emit_expr(inner, map))
        }
        Expr::Call { func, args } => {
            let a: Vec<String> = args
                .iter()
                .map(|arg| match &arg.name {
                    Some(n) => format!("({} {} {})", g(map, "as_alias"), n, emit_expr(&arg.value, map)),
                    None => emit_expr(&arg.value, map),
                })
                .collect();
            format!("({} {} {})", g(map, "call"), emit_expr(func, map), a.join(" "))
        }
        Expr::Attr { obj, name } => format!("({} {} {})", g(map, "attr"), emit_expr(obj, map), name),
        Expr::Sub { obj, idx } => format!("({} {} {})", g(map, "subscript"), emit_expr(obj, map), emit_expr(idx, map)),
        Expr::List(items) => {
            let inner: Vec<String> = items.iter().map(|i| emit_expr(i, map)).collect();
            format!("({} {})", g(map, "list_of"), inner.join(" "))
        }
        Expr::Tuple(items) => {
            let inner: Vec<String> = items.iter().map(|i| emit_expr(i, map)).collect();
            format!("({} {})", g(map, "tuple"), inner.join(" "))
        }
        Expr::Dict(items) => {
            let inner: Vec<String> = items.iter().map(|(k, v)| format!("({} {})", emit_expr(k, map), emit_expr(v, map))).collect();
            format!("({} {})", g(map, "dict_of"), inner.join(" "))
        }
        Expr::Ternary { test, then, els } => format!(
            "({} {} {} {})",
            g(map, "ternary"),
            emit_expr(test, map),
            emit_expr(then, map),
            emit_expr(els, map)
        ),
    }
}

fn emit_param(p: &Param, map: &DialectMap) -> String {
    match p.kind {
        ParamKind::Normal => p.name.clone(),
        ParamKind::Star => format!("({} {})", g(map, "star"), p.name),
        ParamKind::DoubleStar => format!("({} {})", g(map, "dstar"), p.name),
    }
}

fn emit_stmt(s: &Stmt, map: &DialectMap, ind: usize) -> String {
    match s {
        Stmt::Expr(e) => emit_expr(e, map),
        Stmt::Break => format!("{}", g(map, "break")),
        Stmt::Continue => format!("{}", g(map, "continue")),
        Stmt::Pass => format!("{}", g(map, "pass")),
        Stmt::Assign { target, value } => {
            format!("({} {} {})", g(map, "assign"), emit_expr(target, map), emit_expr(value, map))
        }
        Stmt::Aug { op, target, value } => format!(
            "({} {} {} {})",
            g(map, "aug"),
            g(map, match op { BinOp::Add=>"add",BinOp::Sub=>"sub",BinOp::Mul=>"mul",BinOp::Div=>"div",BinOp::FloorDiv=>"floordiv",BinOp::Mod=>"mod",BinOp::Pow=>"pow",BinOp::Eq=>"eq",BinOp::Ne=>"ne",BinOp::Lt=>"lt",BinOp::Gt=>"gt",BinOp::Le=>"le",BinOp::Ge=>"ge",BinOp::And=>"and",BinOp::Or=>"or" }),
            emit_expr(target, map),
            emit_expr(value, map)
        ),
        Stmt::Def { name, params, body } => {
            let ps: Vec<String> = params.iter().map(|p| emit_param(p, map)).collect();
            let params_s = format!("({})", ps.join(" "));
            format!("({} {} {} {})", g(map, "def"), name, params_s, emit_block(body, map, ind))
        }
        Stmt::Ret(None) => format!("({})", g(map, "ret")),
        Stmt::Ret(Some(e)) => format!("({} {})", g(map, "ret"), emit_expr(e, map)),
        Stmt::If { branches } => {
            let bs: Vec<String> = branches
                .iter()
                .map(|(test, body)| match test {
                    Some(t) => format!("({} {})", emit_expr(t, map), emit_block(body, map, ind)),
                    None => format!("(else {})", emit_block(body, map, ind)),
                })
                .collect();
            format!("({} {})", g(map, "if"), bs.join(" "))
        }
        Stmt::While { test, body } => {
            format!("({} {} {})", g(map, "while"), emit_expr(test, map), emit_block(body, map, ind))
        }
        Stmt::For { target, iter, body } => format!(
            "({} {} {} {})",
            g(map, "for"),
            emit_expr(target, map),
            emit_expr(iter, map),
            emit_block(body, map, ind)
        ),
        Stmt::Import { names } => {
            let ns: Vec<String> = names
                .iter()
                .map(|(n, a)| match a {
                    Some(a) => format!("({} {} {})", g(map, "as_alias"), n, a),
                    None => n.clone(),
                })
                .collect();
            format!("({} {})", g(map, "import"), ns.join(" "))
        }
        Stmt::FromImport { module, names } => {
            let ns: Vec<String> = names
                .iter()
                .map(|(n, a)| match a {
                    Some(a) => format!("({} {} {})", g(map, "as_alias"), n, a),
                    None => n.clone(),
                })
                .collect();
            format!("({} {} {} {})", g(map, "from_import"), module, g(map, "import"), ns.join(" "))
        }
    }
}

/// Emit IR as glyph S-expression text (without the header line).
pub fn emit_glyph(stmts: &[Stmt], map: &DialectMap) -> String {
    stmts.iter().map(|s| emit_stmt(s, map, 0)).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::ir::{BinOp, Expr, Param, ParamKind, Stmt};
    use crate::seed::permute::build;
    use crate::seed::prng::from_entropy;

    #[test]
    fn glyph_round_trip() {
        let seed = from_entropy();
        let map = build(&seed);
        let stmts = vec![Stmt::Def {
            name: "add".into(),
            params: vec![
                Param { name: "a".into(), kind: ParamKind::Normal },
                Param { name: "b".into(), kind: ParamKind::Normal },
            ],
            body: vec![Stmt::Ret(Some(Expr::Bin {
                op: BinOp::Add,
                l: Box::new(Expr::Name("a".into())),
                r: Box::new(Expr::Name("b".into())),
            }))],
        }];
        let text = emit_glyph(&stmts, &map);
        let back = parse_glyph(&text, &map).unwrap();
        assert_eq!(stmts, back);
    }
}

