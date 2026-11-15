//! RMojo — a Mojo-style language: Python syntax on the surface, a memory-safe Rust implementation inside (no C++, no `unsafe`).
//! Pipeline: Python source -> rustpython-parser AST -> resolved IR (locals/globals/functions as indices) -> fast interpreter.
//! Supported subset (v0.1): def (typed or untyped), int/float/bool/str, arithmetic, comparisons, and/or/not, if/elif/else, while,
//! for-in-range, break/continue/return, recursion, module-level globals, print/int/float/str/bool/abs/min/max/len.
//! Safety: no `unsafe`; integer overflow is a runtime error; recursion depth and the call stack are bounded.
#![forbid(unsafe_code)]

use rustpython_parser::ast::{self, Constant, Expr, Stmt};
use rustpython_parser::Parse;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

const MAX_DEPTH: usize = 20_000;

#[derive(Clone, Debug)]
enum V {
    I(i64),
    F(f64),
    B(bool),
    S(Rc<String>),
    N,
}

#[derive(Clone, Copy, Debug)]
enum Bop {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
}

#[derive(Clone, Copy, Debug)]
enum Cop {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug)]
enum E {
    Const(V),
    Local(usize),
    Global(usize),
    Bin(Bop, Box<E>, Box<E>),
    Neg(Box<E>),
    Not(Box<E>),
    Cmp(Box<E>, Vec<(Cop, E)>),
    And(Vec<E>),
    Or(Vec<E>),
    If(Box<E>, Box<E>, Box<E>),
    Call(usize, Vec<E>),
    Builtin(&'static str, Vec<E>),
}

#[derive(Clone, Debug)]
enum St {
    SetLocal(usize, E),
    SetGlobal(usize, E),
    Multi(Vec<Target>, Vec<E>),
    Expr(E),
    If(E, Vec<St>, Vec<St>),
    While(E, Vec<St>),
    ForRange(Target, E, E, E, Vec<St>),
    Return(Option<E>),
    Break,
    Continue,
}

#[derive(Clone, Copy, Debug)]
enum Target {
    Local(usize),
    Global(usize),
}

struct Func {
    nparams: usize,
    nlocals: usize,
    body: Vec<St>,
}

struct Program {
    funcs: Vec<Func>,
    main_body: Vec<St>,
    nglobals: usize,
    entry: Option<usize>,
    ifuncs: Vec<Option<IFunc>>,
}

/// Boxed error (keeps `Result<V, Er>` small so the hot path stays in registers).
struct Er(Box<String>);
impl From<&str> for Er {
    fn from(s: &str) -> Er {
        Er(Box::new(s.to_string()))
    }
}
impl From<String> for Er {
    fn from(s: String) -> Er {
        Er(Box::new(s))
    }
}
impl std::fmt::Display for Er {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
type R<T> = Result<T, Er>;

// ---------------------------------------------------------------- compile (AST -> IR)

struct Scope<'a> {
    funcs: &'a HashMap<String, usize>,
    globals: &'a mut HashMap<String, usize>,
    locals: Option<HashMap<String, usize>>, // None at module level
}

impl<'a> Scope<'a> {
    fn lookup_set(&mut self, name: &str) -> Target {
        match &mut self.locals {
            Some(l) => {
                let n = l.len();
                Target::Local(*l.entry(name.to_string()).or_insert(n))
            }
            None => {
                let n = self.globals.len();
                Target::Global(*self.globals.entry(name.to_string()).or_insert(n))
            }
        }
    }
}

fn cexpr(e: &Expr, sc: &mut Scope) -> R<E> {
    Ok(match e {
        Expr::Constant(c) => E::Const(match &c.value {
            Constant::Int(i) => V::I(i.to_string().parse::<i64>().map_err(|_| "integer literal out of range (64-bit)".to_string())?),
            Constant::Float(f) => V::F(*f),
            Constant::Bool(b) => V::B(*b),
            Constant::Str(s) => V::S(Rc::new(s.to_string())),
            Constant::None => V::N,
            _ => return Err("unsupported literal".into()),
        }),
        Expr::Name(n) => {
            let id = n.id.as_str();
            if let Some(l) = &sc.locals {
                if let Some(&i) = l.get(id) {
                    return Ok(E::Local(i));
                }
            }
            if let Some(&g) = sc.globals.get(id) {
                return Ok(E::Global(g));
            }
            return Err(format!("name '{id}' is not defined").into());
        }
        Expr::BinOp(b) => {
            let op = match b.op {
                ast::Operator::Add => Bop::Add,
                ast::Operator::Sub => Bop::Sub,
                ast::Operator::Mult => Bop::Mul,
                ast::Operator::Div => Bop::Div,
                ast::Operator::FloorDiv => Bop::FloorDiv,
                ast::Operator::Mod => Bop::Mod,
                ast::Operator::Pow => Bop::Pow,
                _ => return Err("unsupported operator".into()),
            };
            E::Bin(op, Box::new(cexpr(&b.left, sc)?), Box::new(cexpr(&b.right, sc)?))
        }
        Expr::UnaryOp(u) => match u.op {
            ast::UnaryOp::USub => E::Neg(Box::new(cexpr(&u.operand, sc)?)),
            ast::UnaryOp::UAdd => cexpr(&u.operand, sc)?,
            ast::UnaryOp::Not => E::Not(Box::new(cexpr(&u.operand, sc)?)),
            _ => return Err("unsupported unary operator".into()),
        },
        Expr::Compare(c) => {
            let mut rest = Vec::new();
            for (op, r) in c.ops.iter().zip(c.comparators.iter()) {
                let o = match op {
                    ast::CmpOp::Eq => Cop::Eq,
                    ast::CmpOp::NotEq => Cop::Ne,
                    ast::CmpOp::Lt => Cop::Lt,
                    ast::CmpOp::LtE => Cop::Le,
                    ast::CmpOp::Gt => Cop::Gt,
                    ast::CmpOp::GtE => Cop::Ge,
                    _ => return Err("unsupported comparison".into()),
                };
                rest.push((o, cexpr(r, sc)?));
            }
            E::Cmp(Box::new(cexpr(&c.left, sc)?), rest)
        }
        Expr::BoolOp(b) => {
            let vs = b.values.iter().map(|v| cexpr(v, sc)).collect::<R<Vec<_>>>()?;
            match b.op {
                ast::BoolOp::And => E::And(vs),
                ast::BoolOp::Or => E::Or(vs),
            }
        }
        Expr::IfExp(i) => E::If(Box::new(cexpr(&i.test, sc)?), Box::new(cexpr(&i.body, sc)?), Box::new(cexpr(&i.orelse, sc)?)),
        Expr::Call(c) => {
            let Expr::Name(f) = &*c.func else { return Err("only direct calls of named functions are supported".into()) };
            let args = c.args.iter().map(|a| cexpr(a, sc)).collect::<R<Vec<_>>>()?;
            let name = f.id.as_str();
            if let Some(&fi) = sc.funcs.get(name) {
                return Ok(E::Call(fi, args));
            }
            let b: &'static str = match name {
                "print" => "print",
                "int" => "int",
                "float" => "float",
                "str" => "str",
                "bool" => "bool",
                "abs" => "abs",
                "min" => "min",
                "max" => "max",
                "len" => "len",
                _ => return Err(format!("name '{name}' is not defined").into()),
            };
            E::Builtin(b, args)
        }
        _ => return Err("unsupported expression".into()),
    })
}

fn cblock(body: &[Stmt], sc: &mut Scope) -> R<Vec<St>> {
    let mut out = Vec::new();
    for s in body {
        match s {
            Stmt::Expr(e) => out.push(St::Expr(cexpr(&e.value, sc)?)),
            Stmt::Pass(_) => {}
            Stmt::Break(_) => out.push(St::Break),
            Stmt::Continue(_) => out.push(St::Continue),
            Stmt::Return(r) => out.push(St::Return(match &r.value {
                Some(v) => Some(cexpr(v, sc)?),
                None => None,
            })),
            Stmt::Assign(a) => {
                if a.targets.len() != 1 {
                    return Err("chained assignment is not supported".into());
                }
                if let (Expr::Tuple(tt), Expr::Tuple(vt)) = (&a.targets[0], &*a.value) {
                    if tt.elts.len() != vt.elts.len() {
                        return Err("tuple assignment: length mismatch".into());
                    }
                    let vals = vt.elts.iter().map(|x| cexpr(x, sc)).collect::<R<Vec<_>>>()?;
                    let mut tg = Vec::new();
                    for t in &tt.elts {
                        let Expr::Name(n) = t else { return Err("only simple names can be assigned".into()) };
                        tg.push(sc.lookup_set(n.id.as_str()));
                    }
                    out.push(St::Multi(tg, vals));
                    continue;
                }
                let Expr::Name(n) = &a.targets[0] else { return Err("only simple names can be assigned".into()) };
                let v = cexpr(&a.value, sc)?;
                out.push(match sc.lookup_set(n.id.as_str()) {
                    Target::Local(i) => St::SetLocal(i, v),
                    Target::Global(i) => St::SetGlobal(i, v),
                });
            }
            Stmt::AnnAssign(a) => {
                let Expr::Name(n) = &*a.target else { return Err("only simple names can be assigned".into()) };
                if let Some(v) = &a.value {
                    let v = cexpr(v, sc)?;
                    out.push(match sc.lookup_set(n.id.as_str()) {
                        Target::Local(i) => St::SetLocal(i, v),
                        Target::Global(i) => St::SetGlobal(i, v),
                    });
                } else {
                    sc.lookup_set(n.id.as_str());
                }
            }
            Stmt::AugAssign(a) => {
                let Expr::Name(n) = &*a.target else { return Err("only simple names can be assigned".into()) };
                let op = match a.op {
                    ast::Operator::Add => Bop::Add,
                    ast::Operator::Sub => Bop::Sub,
                    ast::Operator::Mult => Bop::Mul,
                    ast::Operator::Div => Bop::Div,
                    ast::Operator::FloorDiv => Bop::FloorDiv,
                    ast::Operator::Mod => Bop::Mod,
                    ast::Operator::Pow => Bop::Pow,
                    _ => return Err("unsupported operator".into()),
                };
                let cur = cexpr(&Expr::Name(ast::ExprName { range: a.range, id: n.id.clone(), ctx: ast::ExprContext::Load }), sc)?;
                let v = E::Bin(op, Box::new(cur), Box::new(cexpr(&a.value, sc)?));
                out.push(match sc.lookup_set(n.id.as_str()) {
                    Target::Local(i) => St::SetLocal(i, v),
                    Target::Global(i) => St::SetGlobal(i, v),
                });
            }
            Stmt::If(i) => out.push(St::If(cexpr(&i.test, sc)?, cblock(&i.body, sc)?, cblock(&i.orelse, sc)?)),
            Stmt::While(w) => {
                if !w.orelse.is_empty() {
                    return Err("while/else is not supported".into());
                }
                out.push(St::While(cexpr(&w.test, sc)?, cblock(&w.body, sc)?));
            }
            Stmt::For(f) => {
                if !f.orelse.is_empty() {
                    return Err("for/else is not supported".into());
                }
                let Expr::Name(t) = &*f.target else { return Err("for target must be a name".into()) };
                let Expr::Call(c) = &*f.iter else { return Err("only `for x in range(...)` is supported".into()) };
                let is_range = matches!(&*c.func, Expr::Name(n) if n.id.as_str() == "range");
                if !is_range || c.args.is_empty() || c.args.len() > 3 {
                    return Err("only `for x in range(...)` is supported".into());
                }
                let a: Vec<E> = c.args.iter().map(|x| cexpr(x, sc)).collect::<R<Vec<_>>>()?;
                let (start, stop, step) = match a.len() {
                    1 => (E::Const(V::I(0)), a[0].clone(), E::Const(V::I(1))),
                    2 => (a[0].clone(), a[1].clone(), E::Const(V::I(1))),
                    _ => (a[0].clone(), a[1].clone(), a[2].clone()),
                };
                let tgt = sc.lookup_set(t.id.as_str());
                out.push(St::ForRange(tgt, start, stop, step, cblock(&f.body, sc)?));
            }
            _ => return Err("unsupported statement".into()),
        }
    }
    Ok(out)
}

fn compile(src: &str) -> R<Program> {
    let suite = ast::Suite::parse(src, "<rmojo>").map_err(|e| Er::from(format!("SyntaxError: {e}")))?;
    // pass 1: function names
    let mut fnames: HashMap<String, usize> = HashMap::new();
    for s in &suite {
        if let Stmt::FunctionDef(f) = s {
            let n = fnames.len();
            fnames.insert(f.name.to_string(), n);
        }
    }
    let mut globals: HashMap<String, usize> = HashMap::new();
    // pass 2: module-level assignments define globals (so functions can read them); collect in order
    let mut main_stmts: Vec<&Stmt> = Vec::new();
    let mut fdefs: Vec<&ast::StmtFunctionDef> = Vec::new();
    for s in &suite {
        match s {
            Stmt::FunctionDef(f) => fdefs.push(f),
            other => main_stmts.push(other),
        }
    }
    // declare module-level names first (targets of assignments/for)
    fn declare(stmts: &[Stmt], g: &mut HashMap<String, usize>) {
        for s in stmts {
            match s {
                Stmt::Assign(a) => {
                    for t in &a.targets {
                        match t {
                            Expr::Name(n) => {
                                let k = g.len();
                                g.entry(n.id.to_string()).or_insert(k);
                            }
                            Expr::Tuple(tt) => {
                                for e in &tt.elts {
                                    if let Expr::Name(n) = e {
                                        let k = g.len();
                                        g.entry(n.id.to_string()).or_insert(k);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Stmt::AnnAssign(a) => {
                    if let Expr::Name(n) = &*a.target {
                        let k = g.len();
                        g.entry(n.id.to_string()).or_insert(k);
                    }
                }
                Stmt::AugAssign(a) => {
                    if let Expr::Name(n) = &*a.target {
                        let k = g.len();
                        g.entry(n.id.to_string()).or_insert(k);
                    }
                }
                Stmt::For(f) => {
                    if let Expr::Name(n) = &*f.target {
                        let k = g.len();
                        g.entry(n.id.to_string()).or_insert(k);
                    }
                    declare(&f.body, g);
                }
                Stmt::If(i) => {
                    declare(&i.body, g);
                    declare(&i.orelse, g);
                }
                Stmt::While(w) => declare(&w.body, g),
                _ => {}
            }
        }
    }
    let owned: Vec<Stmt> = main_stmts.iter().map(|s| (*s).clone()).collect();
    declare(&owned, &mut globals);

    let mut funcs: Vec<Func> = Vec::new();
    for f in &fdefs {
        let mut locals: HashMap<String, usize> = HashMap::new();
        for a in f.args.args.iter() {
            let n = locals.len();
            locals.insert(a.def.arg.to_string(), n);
        }
        let nparams = locals.len();
        let mut sc = Scope { funcs: &fnames, globals: &mut globals, locals: Some(locals) };
        let body = cblock(&f.body, &mut sc)?;
        let nlocals = sc.locals.as_ref().map(|l| l.len()).unwrap_or(0);
        funcs.push(Func { nparams, nlocals, body });
    }
    let mut sc = Scope { funcs: &fnames, globals: &mut globals, locals: None };
    let main_body = cblock(&owned, &mut sc)?;
    let entry = fnames.get("main").copied();
    // typed int fast path: fixpoint over functions whose signatures are `int -> int` and whose bodies stay inside the int subset
    let mut cand: HashMap<String, usize> = HashMap::new();
    for f in &fdefs {
        if typed_signature(f) {
            cand.insert(f.name.to_string(), fnames[f.name.as_str()]);
        }
    }
    let mut ifuncs: Vec<Option<IFunc>>;
    loop {
        ifuncs = (0..fdefs.len()).map(|_| None).collect();
        let mut removed = false;
        for f in &fdefs {
            if cand.contains_key(f.name.as_str()) {
                match compile_typed(f, &cand) {
                    Some(t) => ifuncs[fnames[f.name.as_str()]] = Some(t),
                    None => {
                        cand.remove(f.name.as_str());
                        removed = true;
                    }
                }
            }
        }
        if !removed {
            break;
        }
    }
    Ok(Program { funcs, main_body, nglobals: globals.len(), entry, ifuncs })
}

// ---------------------------------------------------------------- typed int fast path
// Functions annotated `def f(a: int, ...) -> int` whose bodies only use ints are compiled to an unboxed i64 IR (no tags, no Rc).
// This is the Mojo-style "typed = fast" path. Anything outside the subset silently falls back to the generic interpreter.

#[derive(Debug)]
enum IE {
    C(i64),
    L(usize),
    Bin(Bop, Box<IE>, Box<IE>),
    BinLC(Bop, usize, i64),
    BinLL(Bop, usize, usize),
    Neg(Box<IE>),
    Call(usize, Vec<IE>),
    If(Box<IC>, Box<IE>, Box<IE>),
}
#[derive(Debug)]
enum IC {
    Cmp(Cop, Box<IE>, Box<IE>),
    CmpC(Cop, Box<IE>, i64),
    CmpLC(Cop, usize, i64),
    CmpLL(Cop, usize, usize),
    And(Vec<IC>),
    Or(Vec<IC>),
    Not(Box<IC>),
    Truth(Box<IE>),
}
#[derive(Debug)]
enum IS {
    Set(usize, IE),
    If(IC, Vec<IS>, Vec<IS>),
    While(IC, Vec<IS>),
    For(usize, IE, IE, IE, Vec<IS>),
    Ret(IE),
    Break,
    Continue,
}
#[derive(Debug)]
struct IFunc {
    nparams: usize,
    nlocals: usize,
    body: Vec<IS>,
}

fn mk_bin(op: Bop, a: IE, b: IE) -> IE {
    match (a, b) {
        (IE::L(x), IE::C(c)) => IE::BinLC(op, x, c),
        (IE::L(x), IE::L(y)) => IE::BinLL(op, x, y),
        (a, b) => IE::Bin(op, Box::new(a), Box::new(b)),
    }
}

fn mk_cmp(op: Cop, a: IE, b: IE) -> IC {
    match (a, b) {
        (IE::L(x), IE::C(c)) => IC::CmpLC(op, x, c),
        (IE::L(x), IE::L(y)) => IC::CmpLL(op, x, y),
        (a, IE::C(c)) => IC::CmpC(op, Box::new(a), c),
        (a, b) => IC::Cmp(op, Box::new(a), Box::new(b)),
    }
}

fn is_name(e: &Option<Box<Expr>>, n: &str) -> bool {
    matches!(e.as_deref(), Some(Expr::Name(x)) if x.id.as_str() == n)
}

fn typed_signature(f: &ast::StmtFunctionDef) -> bool {
    f.args.posonlyargs.is_empty()
        && f.args.vararg.is_none()
        && f.args.kwonlyargs.is_empty()
        && f.args.kwarg.is_none()
        && f.args.args.iter().all(|a| a.default.is_none() && is_name(&a.def.annotation, "int"))
        && is_name(&f.returns, "int")
}

struct TCx<'a> {
    typed: &'a HashMap<String, usize>,
    locals: HashMap<String, usize>,
}

fn t_expr(e: &Expr, cx: &TCx) -> Option<IE> {
    Some(match e {
        Expr::Constant(c) => match &c.value {
            Constant::Int(i) => IE::C(i.to_string().parse::<i64>().ok()?),
            _ => return None,
        },
        Expr::Name(n) => IE::L(*cx.locals.get(n.id.as_str())?),
        Expr::BinOp(b) => {
            let op = match b.op {
                ast::Operator::Add => Bop::Add,
                ast::Operator::Sub => Bop::Sub,
                ast::Operator::Mult => Bop::Mul,
                ast::Operator::FloorDiv => Bop::FloorDiv,
                ast::Operator::Mod => Bop::Mod,
                ast::Operator::Pow => Bop::Pow,
                _ => return None,
            };
            mk_bin(op, t_expr(&b.left, cx)?, t_expr(&b.right, cx)?)
        }
        Expr::UnaryOp(u) => match u.op {
            ast::UnaryOp::USub => IE::Neg(Box::new(t_expr(&u.operand, cx)?)),
            ast::UnaryOp::UAdd => t_expr(&u.operand, cx)?,
            _ => return None,
        },
        Expr::IfExp(i) => IE::If(Box::new(t_cond(&i.test, cx)?), Box::new(t_expr(&i.body, cx)?), Box::new(t_expr(&i.orelse, cx)?)),
        Expr::Call(c) => {
            let Expr::Name(f) = &*c.func else { return None };
            let fi = *cx.typed.get(f.id.as_str())?;
            IE::Call(fi, c.args.iter().map(|a| t_expr(a, cx)).collect::<Option<Vec<_>>>()?)
        }
        _ => return None,
    })
}

fn t_cond(e: &Expr, cx: &TCx) -> Option<IC> {
    Some(match e {
        Expr::Compare(c) => {
            let mut parts = Vec::new();
            let mut left = t_expr(&c.left, cx)?;
            for (op, r) in c.ops.iter().zip(c.comparators.iter()) {
                let o = match op {
                    ast::CmpOp::Eq => Cop::Eq,
                    ast::CmpOp::NotEq => Cop::Ne,
                    ast::CmpOp::Lt => Cop::Lt,
                    ast::CmpOp::LtE => Cop::Le,
                    ast::CmpOp::Gt => Cop::Gt,
                    ast::CmpOp::GtE => Cop::Ge,
                    _ => return None,
                };
                let right = t_expr(r, cx)?;
                parts.push(mk_cmp(o, left, right));
                // chained comparison re-evaluates the shared operand: only allowed for names/constants (no side effects)
                if c.ops.len() > 1 && !matches!(r, Expr::Name(_) | Expr::Constant(_)) {
                    return None;
                }
                left = t_expr(r, cx)?;
            }
            if parts.len() == 1 {
                parts.pop()?
            } else {
                IC::And(parts)
            }
        }
        Expr::BoolOp(b) => {
            let vs = b.values.iter().map(|v| t_cond(v, cx)).collect::<Option<Vec<_>>>()?;
            match b.op {
                ast::BoolOp::And => IC::And(vs),
                ast::BoolOp::Or => IC::Or(vs),
            }
        }
        Expr::UnaryOp(u) if matches!(u.op, ast::UnaryOp::Not) => IC::Not(Box::new(t_cond(&u.operand, cx)?)),
        other => IC::Truth(Box::new(t_expr(other, cx)?)),
    })
}

fn t_block(body: &[Stmt], cx: &mut TCx) -> Option<Vec<IS>> {
    let mut out = Vec::new();
    for s in body {
        match s {
            Stmt::Pass(_) => {}
            Stmt::Break(_) => out.push(IS::Break),
            Stmt::Continue(_) => out.push(IS::Continue),
            Stmt::Return(r) => out.push(IS::Ret(t_expr(r.value.as_deref()?, cx)?)),
            Stmt::Assign(a) if a.targets.len() == 1 => {
                let Expr::Name(n) = &a.targets[0] else { return None };
                let v = t_expr(&a.value, cx)?;
                let k = cx.locals.len();
                let i = *cx.locals.entry(n.id.to_string()).or_insert(k);
                out.push(IS::Set(i, v));
            }
            Stmt::AnnAssign(a) => {
                let Expr::Name(n) = &*a.target else { return None };
                if !matches!(&*a.annotation, Expr::Name(x) if x.id.as_str() == "int") {
                    return None;
                }
                let v = t_expr(a.value.as_deref()?, cx)?;
                let k = cx.locals.len();
                let i = *cx.locals.entry(n.id.to_string()).or_insert(k);
                out.push(IS::Set(i, v));
            }
            Stmt::AugAssign(a) => {
                let Expr::Name(n) = &*a.target else { return None };
                let i = *cx.locals.get(n.id.as_str())?;
                let op = match a.op {
                    ast::Operator::Add => Bop::Add,
                    ast::Operator::Sub => Bop::Sub,
                    ast::Operator::Mult => Bop::Mul,
                    ast::Operator::FloorDiv => Bop::FloorDiv,
                    ast::Operator::Mod => Bop::Mod,
                    ast::Operator::Pow => Bop::Pow,
                    _ => return None,
                };
                out.push(IS::Set(i, mk_bin(op, IE::L(i), t_expr(&a.value, cx)?)));
            }
            Stmt::If(i) => {
                let c = t_cond(&i.test, cx)?;
                let a = t_block(&i.body, cx)?;
                let b = t_block(&i.orelse, cx)?;
                out.push(IS::If(c, a, b));
            }
            Stmt::While(w) if w.orelse.is_empty() => {
                let c = t_cond(&w.test, cx)?;
                out.push(IS::While(c, t_block(&w.body, cx)?));
            }
            Stmt::For(f) if f.orelse.is_empty() => {
                let Expr::Name(t) = &*f.target else { return None };
                let Expr::Call(c) = &*f.iter else { return None };
                if !matches!(&*c.func, Expr::Name(n) if n.id.as_str() == "range") || c.args.is_empty() || c.args.len() > 3 {
                    return None;
                }
                let a: Vec<IE> = c.args.iter().map(|x| t_expr(x, cx)).collect::<Option<Vec<_>>>()?;
                let mut it = a.into_iter();
                let (start, stop, step) = match it.len() {
                    1 => (IE::C(0), it.next()?, IE::C(1)),
                    2 => (it.next()?, it.next()?, IE::C(1)),
                    _ => (it.next()?, it.next()?, it.next()?),
                };
                let k = cx.locals.len();
                let ti = *cx.locals.entry(t.id.to_string()).or_insert(k);
                out.push(IS::For(ti, start, stop, step, t_block(&f.body, cx)?));
            }
            _ => return None,
        }
    }
    Some(out)
}

fn compile_typed(f: &ast::StmtFunctionDef, typed: &HashMap<String, usize>) -> Option<IFunc> {
    let mut locals = HashMap::new();
    for a in f.args.args.iter() {
        let n = locals.len();
        locals.insert(a.def.arg.to_string(), n);
    }
    let nparams = locals.len();
    let mut cx = TCx { typed, locals };
    let body = t_block(&f.body, &mut cx)?;
    Some(IFunc { nparams, nlocals: cx.locals.len(), body })
}

enum IFlow {
    Next,
    Break,
    Continue,
    Ret(i64),
}

fn iovf() -> Er {
    Er::from("OverflowError: integer overflow (RMojo integers are 64-bit)")
}

fn ibin(op: Bop, x: i64, y: i64) -> R<i64> {
    match op {
        Bop::Add => x.checked_add(y).ok_or_else(iovf),
        Bop::Sub => x.checked_sub(y).ok_or_else(iovf),
        Bop::Mul => x.checked_mul(y).ok_or_else(iovf),
        Bop::FloorDiv => {
            if y == 0 {
                return Err("ZeroDivisionError: integer division or modulo by zero".into());
            }
            let q = x.checked_div(y).ok_or_else(iovf)?;
            Ok(if (x % y != 0) && ((x < 0) != (y < 0)) { q - 1 } else { q })
        }
        Bop::Mod => {
            if y == 0 {
                return Err("ZeroDivisionError: integer division or modulo by zero".into());
            }
            let r = x.checked_rem(y).unwrap_or(0);
            Ok(if r != 0 && ((r < 0) != (y < 0)) { r + y } else { r })
        }
        Bop::Pow => {
            if y < 0 {
                return Err("TypeError: negative exponent needs float (not supported in typed int functions)".into());
            }
            x.checked_pow(u32::try_from(y).map_err(|_| iovf())?).ok_or_else(iovf)
        }
        Bop::Div => Err("internal: / in typed function".into()),
    }
}

#[inline(always)]
fn cmp_i(op: Cop, x: i64, y: i64) -> bool {
    match op {
        Cop::Eq => x == y,
        Cop::Ne => x != y,
        Cop::Lt => x < y,
        Cop::Le => x <= y,
        Cop::Gt => x > y,
        Cop::Ge => x >= y,
    }
}

struct IVm<'p> {
    funcs: &'p [Option<IFunc>],
    stack: Vec<i64>,
    depth: usize,
}

impl<'p> IVm<'p> {
    #[inline]
    fn ev(&mut self, e: &IE, base: usize) -> R<i64> {
        match e {
            IE::C(v) => Ok(*v),
            IE::L(i) => Ok(self.stack[base + *i]),
            IE::Bin(op, a, b) => {
                let x = self.ev(a, base)?;
                let y = self.ev(b, base)?;
                match op {
                    Bop::Add => x.checked_add(y).ok_or_else(iovf),
                    Bop::Sub => x.checked_sub(y).ok_or_else(iovf),
                    Bop::Mul => x.checked_mul(y).ok_or_else(iovf),
                    _ => ibin(*op, x, y),
                }
            }
            IE::BinLC(op, i, c) => {
                let x = self.stack[base + *i];
                match op {
                    Bop::Add => x.checked_add(*c).ok_or_else(iovf),
                    Bop::Sub => x.checked_sub(*c).ok_or_else(iovf),
                    Bop::Mul => x.checked_mul(*c).ok_or_else(iovf),
                    Bop::Mod if *c > 0 => Ok(x.rem_euclid(*c)),
                    Bop::FloorDiv if *c > 0 => Ok(x.div_euclid(*c)),
                    _ => ibin(*op, x, *c),
                }
            }
            IE::BinLL(op, i, j) => {
                let (x, y) = (self.stack[base + *i], self.stack[base + *j]);
                match op {
                    Bop::Add => x.checked_add(y).ok_or_else(iovf),
                    Bop::Sub => x.checked_sub(y).ok_or_else(iovf),
                    Bop::Mul => x.checked_mul(y).ok_or_else(iovf),
                    _ => ibin(*op, x, y),
                }
            }
            IE::Neg(a) => self.ev(a, base)?.checked_neg().ok_or_else(iovf),
            IE::If(c, a, b) => {
                if self.cd(c, base)? {
                    self.ev(a, base)
                } else {
                    self.ev(b, base)
                }
            }
            IE::Call(fi, args) => {
                let nbase = self.stack.len();
                for a in args {
                    let v = self.ev(a, base)?;
                    self.stack.push(v);
                }
                self.run(*fi, nbase)
            }
        }
    }

    fn run(&mut self, fi: usize, nbase: usize) -> R<i64> {
        let Some(f) = self.funcs[fi].as_ref() else { return Err("internal: not a typed function".into()) };
        if self.depth >= MAX_DEPTH {
            return Err("RecursionError: maximum recursion depth exceeded".into());
        }
        self.stack.resize(nbase + f.nlocals, 0);
        self.depth += 1;
        let r = self.blk(&f.body, nbase);
        self.depth -= 1;
        self.stack.truncate(nbase);
        match r? {
            IFlow::Ret(v) => Ok(v),
            _ => Err("TypeError: function annotated -> int returned no value".into()),
        }
    }

    fn cd(&mut self, c: &IC, base: usize) -> R<bool> {
        Ok(match c {
            IC::Cmp(op, a, b) => {
                let x = self.ev(a, base)?;
                let y = self.ev(b, base)?;
                match op {
                    Cop::Eq => x == y,
                    Cop::Ne => x != y,
                    Cop::Lt => x < y,
                    Cop::Le => x <= y,
                    Cop::Gt => x > y,
                    Cop::Ge => x >= y,
                }
            }
            IC::CmpC(op, a, c) => cmp_i(*op, self.ev(a, base)?, *c),
            IC::CmpLC(op, i, c) => cmp_i(*op, self.stack[base + *i], *c),
            IC::CmpLL(op, i, j) => cmp_i(*op, self.stack[base + *i], self.stack[base + *j]),
            IC::And(v) => {
                for c in v {
                    if !self.cd(c, base)? {
                        return Ok(false);
                    }
                }
                true
            }
            IC::Or(v) => {
                for c in v {
                    if self.cd(c, base)? {
                        return Ok(true);
                    }
                }
                false
            }
            IC::Not(c) => !self.cd(c, base)?,
            IC::Truth(e) => self.ev(e, base)? != 0,
        })
    }

    fn blk(&mut self, body: &[IS], base: usize) -> R<IFlow> {
        for s in body {
            match s {
                IS::Set(i, e) => {
                    let v = self.ev(e, base)?;
                    self.stack[base + *i] = v;
                }
                IS::If(c, a, b) => {
                    let f = if self.cd(c, base)? { self.blk(a, base)? } else { self.blk(b, base)? };
                    if !matches!(f, IFlow::Next) {
                        return Ok(f);
                    }
                }
                IS::While(c, b) => {
                    while self.cd(c, base)? {
                        match self.blk(b, base)? {
                            IFlow::Break => break,
                            IFlow::Ret(v) => return Ok(IFlow::Ret(v)),
                            _ => {}
                        }
                    }
                }
                IS::For(t, start, stop, step, b) => {
                    let mut i = self.ev(start, base)?;
                    let end = self.ev(stop, base)?;
                    let st = self.ev(step, base)?;
                    if st == 0 {
                        return Err("ValueError: range() arg 3 must not be zero".into());
                    }
                    while (st > 0 && i < end) || (st < 0 && i > end) {
                        self.stack[base + *t] = i;
                        match self.blk(b, base)? {
                            IFlow::Break => break,
                            IFlow::Ret(v) => return Ok(IFlow::Ret(v)),
                            _ => {}
                        }
                        i = match i.checked_add(st) {
                            Some(n) => n,
                            None => break,
                        };
                    }
                }
                IS::Ret(e) => return Ok(IFlow::Ret(self.ev(e, base)?)),
                IS::Break => return Ok(IFlow::Break),
                IS::Continue => return Ok(IFlow::Continue),
            }
        }
        Ok(IFlow::Next)
    }
}

// ---------------------------------------------------------------- run

enum Flow {
    Next,
    Break,
    Continue,
    Return(V),
}

struct Vm<'p> {
    p: &'p Program,
    globals: Vec<V>,
    depth: usize,
    stack: Vec<V>,
    out: std::io::BufWriter<std::io::Stdout>,
}

fn pyrepr_f(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    let a = f.abs();
    if a != 0.0 && !(1e-4..1e16).contains(&a) {
        let s = format!("{:e}", f); // e.g. 1e21, 1.5e-7
        let (m, e) = s.split_once('e').unwrap_or((&s, "0"));
        let ev: i32 = e.parse().unwrap_or(0);
        return format!("{}e{}{:02}", m, if ev < 0 { '-' } else { '+' }, ev.abs());
    }
    let s = format!("{:?}", f);
    s
}

fn show(v: &V) -> String {
    match v {
        V::I(i) => i.to_string(),
        V::F(f) => pyrepr_f(*f),
        V::B(b) => if *b { "True".into() } else { "False".into() },
        V::S(s) => s.as_str().to_string(),
        V::N => "None".into(),
    }
}

fn truthy(v: &V) -> bool {
    match v {
        V::I(i) => *i != 0,
        V::F(f) => *f != 0.0,
        V::B(b) => *b,
        V::S(s) => !s.is_empty(),
        V::N => false,
    }
}

fn num(v: &V) -> Option<f64> {
    match v {
        V::I(i) => Some(*i as f64),
        V::F(f) => Some(*f),
        V::B(b) => Some(*b as i64 as f64),
        _ => None,
    }
}

fn int_of(v: &V) -> Option<i64> {
    match v {
        V::I(i) => Some(*i),
        V::B(b) => Some(*b as i64),
        _ => None,
    }
}

fn binop(op: Bop, a: V, b: V) -> R<V> {
    if let (Some(x), Some(y)) = (int_of(&a), int_of(&b)) {
        let ovf = || "OverflowError: integer overflow (RMojo integers are 64-bit)".to_string();
        return Ok(match op {
            Bop::Add => V::I(x.checked_add(y).ok_or_else(ovf)?),
            Bop::Sub => V::I(x.checked_sub(y).ok_or_else(ovf)?),
            Bop::Mul => V::I(x.checked_mul(y).ok_or_else(ovf)?),
            Bop::Div => {
                if y == 0 {
                    return Err("ZeroDivisionError: division by zero".into());
                }
                V::F(x as f64 / y as f64)
            }
            Bop::FloorDiv => {
                if y == 0 {
                    return Err("ZeroDivisionError: integer division or modulo by zero".into());
                }
                let q = x.checked_div(y).ok_or_else(ovf)?;
                V::I(if (x % y != 0) && ((x < 0) != (y < 0)) { q - 1 } else { q })
            }
            Bop::Mod => {
                if y == 0 {
                    return Err("ZeroDivisionError: integer division or modulo by zero".into());
                }
                let r = x.checked_rem(y).unwrap_or(0);
                V::I(if r != 0 && ((r < 0) != (y < 0)) { r + y } else { r })
            }
            Bop::Pow => {
                if y >= 0 {
                    let e = u32::try_from(y).map_err(|_| ovf())?;
                    V::I(x.checked_pow(e).ok_or_else(ovf)?)
                } else {
                    V::F((x as f64).powf(y as f64))
                }
            }
        });
    }
    if let (V::S(x), V::S(y), Bop::Add) = (&a, &b, op) {
        return Ok(V::S(Rc::new(format!("{x}{y}"))));
    }
    if let (Some(x), Some(y)) = (num(&a), num(&b)) {
        return Ok(match op {
            Bop::Add => V::F(x + y),
            Bop::Sub => V::F(x - y),
            Bop::Mul => V::F(x * y),
            Bop::Div => {
                if y == 0.0 {
                    return Err("ZeroDivisionError: float division by zero".into());
                }
                V::F(x / y)
            }
            Bop::FloorDiv => {
                if y == 0.0 {
                    return Err("ZeroDivisionError: float floor division by zero".into());
                }
                V::F((x / y).floor())
            }
            Bop::Mod => {
                if y == 0.0 {
                    return Err("ZeroDivisionError: float modulo".into());
                }
                let r = x % y;
                V::F(if r != 0.0 && ((r < 0.0) != (y < 0.0)) { r + y } else { r })
            }
            Bop::Pow => V::F(x.powf(y)),
        });
    }
    Err("TypeError: unsupported operand types".into())
}

fn compare(op: Cop, a: &V, b: &V) -> R<bool> {
    if let (V::S(x), V::S(y)) = (a, b) {
        return Ok(match op {
            Cop::Eq => x == y,
            Cop::Ne => x != y,
            Cop::Lt => x < y,
            Cop::Le => x <= y,
            Cop::Gt => x > y,
            Cop::Ge => x >= y,
        });
    }
    if let (V::N, V::N) = (a, b) {
        return Ok(matches!(op, Cop::Eq | Cop::Le | Cop::Ge));
    }
    if let (Some(x), Some(y)) = (int_of(a), int_of(b)) {
        return Ok(match op {
            Cop::Eq => x == y,
            Cop::Ne => x != y,
            Cop::Lt => x < y,
            Cop::Le => x <= y,
            Cop::Gt => x > y,
            Cop::Ge => x >= y,
        });
    }
    if let (Some(x), Some(y)) = (num(a), num(b)) {
        return Ok(match op {
            Cop::Eq => x == y,
            Cop::Ne => x != y,
            Cop::Lt => x < y,
            Cop::Le => x <= y,
            Cop::Gt => x > y,
            Cop::Ge => x >= y,
        });
    }
    match op {
        Cop::Eq => Ok(false),
        Cop::Ne => Ok(true),
        _ => Err("TypeError: unorderable types".into()),
    }
}

impl<'p> Vm<'p> {
    /// Condition evaluation without materialising a `V` for the common int comparison.
    #[inline]
    fn cond(&mut self, e: &E, base: usize) -> R<bool> {
        if let E::Cmp(first, rest) = e {
            if rest.len() == 1 {
                let l = self.eval(first, base)?;
                let r = self.eval(&rest[0].1, base)?;
                if let (V::I(p), V::I(q)) = (&l, &r) {
                    return Ok(match rest[0].0 {
                        Cop::Eq => p == q,
                        Cop::Ne => p != q,
                        Cop::Lt => p < q,
                        Cop::Le => p <= q,
                        Cop::Gt => p > q,
                        Cop::Ge => p >= q,
                    });
                }
                return compare(rest[0].0, &l, &r);
            }
        }
        Ok(truthy(&self.eval(e, base)?))
    }

    #[inline]
    fn eval(&mut self, e: &E, base: usize) -> R<V> {
        Ok(match e {
            E::Const(v) => v.clone(),
            E::Local(i) => self.stack[base + *i].clone(),
            E::Global(i) => self.globals[*i].clone(),
            E::Bin(op, a, b) => {
                let x = self.eval(a, base)?;
                let y = self.eval(b, base)?;
                if let (V::I(p), V::I(q)) = (&x, &y) {
                    let r = match op {
                        Bop::Add => p.checked_add(*q),
                        Bop::Sub => p.checked_sub(*q),
                        Bop::Mul => p.checked_mul(*q),
                        _ => None,
                    };
                    if let Some(r) = r {
                        return Ok(V::I(r));
                    }
                }
                binop(*op, x, y)?
            }
            E::Neg(a) => match self.eval(a, base)? {
                V::I(i) => V::I(i.checked_neg().ok_or("OverflowError: integer overflow")?),
                V::F(f) => V::F(-f),
                V::B(b) => V::I(-(b as i64)),
                _ => return Err("TypeError: bad operand type for unary -".into()),
            },
            E::Not(a) => V::B(!truthy(&self.eval(a, base)?)),
            E::Cmp(first, rest) => {
                if rest.len() == 1 {
                    let l = self.eval(first, base)?;
                    let r = self.eval(&rest[0].1, base)?;
                    if let (V::I(p), V::I(q)) = (&l, &r) {
                        return Ok(V::B(match rest[0].0 {
                            Cop::Eq => p == q,
                            Cop::Ne => p != q,
                            Cop::Lt => p < q,
                            Cop::Le => p <= q,
                            Cop::Gt => p > q,
                            Cop::Ge => p >= q,
                        }));
                    }
                    return Ok(V::B(compare(rest[0].0, &l, &r)?));
                }
                let mut l = self.eval(first, base)?;
                let mut ok = true;
                for (op, r) in rest {
                    let rv = self.eval(r, base)?;
                    if !compare(*op, &l, &rv)? {
                        ok = false;
                        break;
                    }
                    l = rv;
                }
                V::B(ok)
            }
            E::And(vs) => {
                let mut last = V::B(true);
                for v in vs {
                    last = self.eval(v, base)?;
                    if !truthy(&last) {
                        break;
                    }
                }
                last
            }
            E::Or(vs) => {
                let mut last = V::B(false);
                for v in vs {
                    last = self.eval(v, base)?;
                    if truthy(&last) {
                        break;
                    }
                }
                last
            }
            E::If(c, a, b) => {
                if self.cond(c, base)? {
                    self.eval(a, base)?
                } else {
                    self.eval(b, base)?
                }
            }
            E::Call(fi, args) => {
                let f = &self.p.funcs[*fi];
                if args.len() != f.nparams {
                    return Err(format!("TypeError: function takes {} arguments but {} were given", f.nparams, args.len()).into());
                }
                if self.depth >= MAX_DEPTH {
                    return Err("RecursionError: maximum recursion depth exceeded".into());
                }
                if self.p.ifuncs[*fi].is_some() {
                    let mut iv = IVm { funcs: &self.p.ifuncs, stack: Vec::with_capacity(64), depth: self.depth };
                    for a in args.iter() {
                        let v = match self.eval(a, base)? {
                            V::I(i) => i,
                            V::B(b) => b as i64,
                            _ => return Err("TypeError: argument must be int".into()),
                        };
                        iv.stack.push(v);
                    }
                    let r = iv.run(*fi, 0)?;
                    return Ok(V::I(r));
                }
                let nbase = self.stack.len();
                for a in args.iter() {
                    let v = self.eval(a, base)?;
                    self.stack.push(v);
                }
                self.stack.resize(nbase + f.nlocals, V::N);
                self.depth += 1;
                let r = self.exec_block(&f.body, nbase);
                self.depth -= 1;
                self.stack.truncate(nbase);
                match r? {
                    Flow::Return(v) => v,
                    _ => V::N,
                }
            }
            E::Builtin(name, args) => {
                let vals = args.iter().map(|a| self.eval(a, base)).collect::<R<Vec<_>>>()?;
                self.builtin(name, vals)?
            }
        })
    }

    fn builtin(&mut self, name: &str, a: Vec<V>) -> R<V> {
        Ok(match name {
            "print" => {
                let s = a.iter().map(show).collect::<Vec<_>>().join(" ");
                writeln!(self.out, "{s}").map_err(|e| Er::from(e.to_string()))?;
                V::N
            }
            "int" => match a.first() {
                Some(V::F(f)) if f.is_finite() && f.abs() < 9.2e18 => V::I(f.trunc() as i64),
                Some(V::I(i)) => V::I(*i),
                Some(V::B(b)) => V::I(*b as i64),
                Some(V::S(s)) => V::I(s.trim().parse::<i64>().map_err(|_| format!("ValueError: invalid literal for int(): '{s}'"))?),
                _ => return Err("TypeError: int() argument".into()),
            },
            "float" => match a.first() {
                Some(v) if num(v).is_some() => V::F(num(v).unwrap_or(0.0)),
                Some(V::S(s)) => V::F(s.trim().parse::<f64>().map_err(|_| format!("ValueError: could not convert string to float: '{s}'"))?),
                _ => return Err("TypeError: float() argument".into()),
            },
            "str" => V::S(Rc::new(show(a.first().unwrap_or(&V::N)))),
            "bool" => V::B(truthy(a.first().unwrap_or(&V::B(false)))),
            "abs" => match a.first() {
                Some(V::I(i)) => V::I(i.checked_abs().ok_or("OverflowError: integer overflow")?),
                Some(V::F(f)) => V::F(f.abs()),
                _ => return Err("TypeError: abs() argument".into()),
            },
            "len" => match a.first() {
                Some(V::S(s)) => V::I(s.chars().count() as i64),
                _ => return Err("TypeError: len() argument".into()),
            },
            "min" | "max" => {
                if a.is_empty() {
                    return Err(format!("TypeError: {name}() expected arguments").into());
                }
                let mut best = a[0].clone();
                for v in &a[1..] {
                    let better = if name == "min" { compare(Cop::Lt, v, &best)? } else { compare(Cop::Gt, v, &best)? };
                    if better {
                        best = v.clone();
                    }
                }
                best
            }
            _ => return Err(format!("unknown builtin {name}").into()),
        })
    }

    fn exec_block(&mut self, body: &[St], base: usize) -> R<Flow> {
        for s in body {
            match s {
                St::SetLocal(i, e) => {
                    let v = self.eval(e, base)?;
                    self.stack[base + *i] = v;
                }
                St::SetGlobal(i, e) => {
                    let v = self.eval(e, base)?;
                    self.globals[*i] = v;
                }
                St::Multi(ts, es) => {
                    let mut vals = Vec::with_capacity(es.len());
                    for e in es {
                        vals.push(self.eval(e, base)?);
                    }
                    for (t, v) in ts.iter().zip(vals) {
                        match t {
                            Target::Local(k) => self.stack[base + *k] = v,
                            Target::Global(k) => self.globals[*k] = v,
                        }
                    }
                }
                St::Expr(e) => {
                    self.eval(e, base)?;
                }
                St::If(c, a, b) => {
                    let f = if self.cond(c, base)? { self.exec_block(a, base)? } else { self.exec_block(b, base)? };
                    if !matches!(f, Flow::Next) {
                        return Ok(f);
                    }
                }
                St::While(c, b) => {
                    while self.cond(c, base)? {
                        match self.exec_block(b, base)? {
                            Flow::Break => break,
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            _ => {}
                        }
                    }
                }
                St::ForRange(t, start, stop, step, b) => {
                    let (Some(mut i), Some(end), Some(st)) =
                        (int_of(&self.eval(start, base)?), int_of(&self.eval(stop, base)?), int_of(&self.eval(step, base)?))
                    else {
                        return Err("TypeError: range() arguments must be integers".into());
                    };
                    if st == 0 {
                        return Err("ValueError: range() arg 3 must not be zero".into());
                    }
                    while (st > 0 && i < end) || (st < 0 && i > end) {
                        match t {
                            Target::Local(k) => self.stack[base + *k] = V::I(i),
                            Target::Global(k) => self.globals[*k] = V::I(i),
                        }
                        match self.exec_block(b, base)? {
                            Flow::Break => break,
                            Flow::Return(v) => return Ok(Flow::Return(v)),
                            _ => {}
                        }
                        i = match i.checked_add(st) {
                            Some(n) => n,
                            None => break,
                        };
                    }
                }
                St::Return(e) => {
                    let v = match e {
                        Some(e) => self.eval(e, base)?,
                        None => V::N,
                    };
                    return Ok(Flow::Return(v));
                }
                St::Break => return Ok(Flow::Break),
                St::Continue => return Ok(Flow::Continue),
            }
        }
        Ok(Flow::Next)
    }
}

fn run_source(src: &str) -> R<()> {
    let prog = compile(src)?;
    let mut vm = Vm { p: &prog, globals: vec![V::N; prog.nglobals], depth: 0, stack: Vec::new(), out: std::io::BufWriter::new(std::io::stdout()) };
    let r = vm.exec_block(&prog.main_body, 0).map(|_| ());
    let r = r.and_then(|_| match prog.entry {
        Some(fi) => {
            let f = &prog.funcs[fi];
            vm.stack.resize(f.nlocals, V::N);
            vm.exec_block(&f.body, 0).map(|_| ())
        }
        None => Ok(()),
    });
    let _ = vm.out.flush();
    r
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--version") | Some("-V") => println!("rmojo 0.1.0 (Rust, no C++)"),
        Some("run") if args.len() > 2 => {
            let src = match std::fs::read_to_string(&args[2]) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("rmojo: cannot read {}: {e}", args[2]);
                    std::process::exit(2);
                }
            };
            // run on a big stack so deep (but bounded) recursion is safe
            let h = std::thread::Builder::new().stack_size(1 << 28).spawn(move || run_source(&src));
            match h.map(|h| h.join()) {
                Ok(Ok(Ok(()))) => {}
                Ok(Ok(Err(e))) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
                _ => {
                    eprintln!("rmojo: internal error");
                    std::process::exit(70);
                }
            }
        }
        _ => {
            eprintln!("usage: rmojo run <file.mojo|file.py> | rmojo --version");
            std::process::exit(2);
        }
    }
}
