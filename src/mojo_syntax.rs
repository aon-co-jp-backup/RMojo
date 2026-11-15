//! Mojo surface syntax -> the Python subset RMojo runs. Line based, deliberately small and honest.
//! Supported: `fn` (-> `def`, `raises` dropped), `var`/`let`, `alias X = v`, argument conventions (`inout`/`borrowed`/`owned`/`mut`/`read`/`out`),
//! Mojo type names (Int, Float64, String, Bool ...), `struct` (-> `class`; fields from `var`; conformance lists dropped),
//! `trait` blocks (dropped: RMojo is duck typed, conformance is not checked), decorators `@value`/`@always_inline`/... (dropped),
//! `List[T](...)` / `Dict[K, V]()` / `SIMD[T, n](...)` constructors.
//! NOT supported (reported as an error, never silently mis-run): other decorators, parametric `fn f[T: ...]`, `import` of Mojo modules, pointers.

fn map_type_words(line: &str) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let flush = |w: &mut String, out: &mut String| {
        if !w.is_empty() {
            let m = match w.as_str() {
                "Int" | "Int8" | "Int16" | "Int32" | "Int64" | "UInt" | "UInt8" | "UInt16" | "UInt32" | "UInt64" => "int",
                "Float16" | "Float32" | "Float64" => "float",
                "String" | "StringLiteral" => "str",
                "Bool" => "bool",
                other => other,
            };
            out.push_str(m);
            w.clear();
        }
    };
    let mut in_str: Option<char> = None;
    for c in line.chars() {
        if let Some(q) = in_str {
            out.push(c);
            if c == q {
                in_str = None;
            }
            continue;
        }
        if c == '"' || c == '\'' {
            flush(&mut word, &mut out);
            in_str = Some(c);
            out.push(c);
        } else if c.is_alphanumeric() || c == '_' {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

fn drop_conventions(s: &str) -> String {
    let mut t = s.to_string();
    for kw in ["inout ", "borrowed ", "owned ", "mut ", "read ", "out ", "deinit "] {
        t = t.replace(&format!("({kw}"), "(").replace(&format!(", {kw}"), ", ");
    }
    t
}

/// `List[T](a, b)` -> `[a, b]`, `Dict[K, V]()` -> `{}`, `SIMD[T, n](a, b)` -> `simd(a, b)`. Type uses without a call are left alone.
fn rewrite_ctors(line: &str) -> String {
    let cs: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        let prev_ident = i > 0 && (cs[i - 1].is_alphanumeric() || cs[i - 1] == '_' || cs[i - 1] == '.');
        let rest: String = cs[i..].iter().take(5).collect();
        let kind = if prev_ident {
            None
        } else if rest.starts_with("List[") {
            Some(("List", 4))
        } else if rest.starts_with("Dict[") {
            Some(("Dict", 4))
        } else if rest.starts_with("SIMD[") {
            Some(("SIMD", 4))
        } else {
            None
        };
        if let Some((k, nlen)) = kind {
            // balanced [...]
            let mut j = i + nlen;
            let mut depth = 0;
            while j < cs.len() {
                if cs[j] == '[' {
                    depth += 1;
                } else if cs[j] == ']' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            if j < cs.len() && j + 1 < cs.len() && cs[j + 1] == '(' {
                // balanced (...)
                let mut m = j + 1;
                let mut pd = 0;
                while m < cs.len() {
                    if cs[m] == '(' {
                        pd += 1;
                    } else if cs[m] == ')' {
                        pd -= 1;
                        if pd == 0 {
                            break;
                        }
                    }
                    m += 1;
                }
                if m < cs.len() {
                    let inner: String = cs[j + 2..m].iter().collect();
                    let inner = rewrite_ctors(&inner);
                    match k {
                        "List" => out.push_str(&format!("[{inner}]")),
                        "Dict" => out.push_str("{}"),
                        _ => out.push_str(&format!("simd({inner})")),
                    }
                    i = m + 1;
                    continue;
                }
            }
        }
        out.push(cs[i]);
        i += 1;
    }
    out
}

/// Drop `[T: Trait, ...]` parameter lists (RMojo is duck typed): `fn f[T: X](..)`, `struct S[T: X]:`, and explicit
/// instantiations such as `f[Int](x)` / `Pair[Int, String](1, "a")` (bracket content starting with an upper-case type word).
fn strip_params(line: &str, decl: bool) -> String {
    let cs: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '[' && i > 0 && (cs[i - 1].is_alphanumeric() || cs[i - 1] == '_') {
            let mut j = i;
            let mut d = 0;
            while j < cs.len() {
                if cs[j] == '[' {
                    d += 1;
                } else if cs[j] == ']' {
                    d -= 1;
                    if d == 0 {
                        break;
                    }
                }
                j += 1;
            }
            let upper = cs.get(i + 1).map(|c| c.is_uppercase()).unwrap_or(false);
            let call_next = cs.get(j + 1) == Some(&'(');
            if j < cs.len() && ((decl && (call_next || cs.get(j + 1) == Some(&':'))) || (upper && call_next)) {
                i = j + 1;
                continue;
            }
        }
        out.push(cs[i]);
        i += 1;
    }
    out
}

pub fn to_python(src: &str) -> Result<String, String> {
    let mut out = Vec::new();
    let mut struct_indent: Option<usize> = None;
    let mut trait_indent: Option<usize> = None;
    for (n, raw) in src.lines().enumerate() {
        let ind: String = raw.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
        let body = &raw[ind.len()..];
        let t = body.trim_start();
        let unsupported = |what: &str| Err(format!("rmojo: line {}: Mojo `{what}` is not supported yet", n + 1));
        if !t.is_empty() {
            if let Some(si) = struct_indent {
                if ind.len() <= si {
                    struct_indent = None;
                }
            }
            if let Some(ti) = trait_indent {
                if ind.len() <= ti {
                    trait_indent = None;
                } else {
                    continue; // inside a trait block: dropped
                }
            }
        } else if trait_indent.is_some() {
            continue;
        }
        if t.starts_with("from ") && t.contains(" import ") || t.starts_with("import ") {
            continue; // Mojo std imports: the supported names are builtins in RMojo
        }
        if t.starts_with("trait ") {
            trait_indent = Some(ind.len());
            continue;
        }
        if t.starts_with('@') {
            let d = t.trim_start_matches('@').split(['(', ' ']).next().unwrap_or("");
            if matches!(d, "value" | "always_inline" | "no_inline" | "register_passable" | "staticmethod" | "nonmaterializable" | "parameter" | "export") {
                continue;
            }
            return unsupported(&format!("@{d}"));
        }
        if let Some(rest) = t.strip_prefix("struct ") {
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
            struct_indent = Some(ind.len());
            if name.is_empty() {
                return unsupported("struct");
            }
            out.push(format!("{ind}class {name}:"));
            continue;
        }
        let in_struct = struct_indent.is_some();
        let mut line = body.to_string();
        if let Some(rest) = t.strip_prefix("except ") {
            // Mojo: `except e:` binds the error; Python: `except Type as e:`
            let r = rest.trim_end_matches(':').trim();
            if !r.is_empty() && !r.contains(' ') && !r.contains('(') {
                line = format!("except Exception as {r}:");
            }
        }
        if t.starts_with("fn ") || t.starts_with("def ") {
            line = strip_params(&line, true);
            line = line.replacen("fn ", "def ", 1);
            line = drop_conventions(&line);
            line = line.replace(" raises:", ":").replace(" raises ->", " ->");
        } else if let Some(rest) = t.strip_prefix("var ").or_else(|| t.strip_prefix("let ")) {
            line = rest.to_string();
            if !line.contains('=') && !in_struct {
                return unsupported("declaration without a value");
            }
        } else if let Some(rest) = t.strip_prefix("alias ") {
            line = rest.to_string();
        }
        out.push(format!("{ind}{}", map_type_words(&strip_params(&rewrite_ctors(&line), false))));
    }
    Ok(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fn_var_types() {
        let s = "fn add(x: Int, y: Int) -> Int:\n    var z = x + y\n    return z\n";
        assert_eq!(to_python(s).unwrap(), "def add(x: int, y: int) -> int:\n    z = x + y\n    return z");
    }
    #[test]
    fn conventions_and_raises() {
        assert_eq!(to_python("fn f(inout a: Int, borrowed b: Float64) raises -> Int:").unwrap(), "def f(a: int, b: float) -> int:");
    }
    #[test]
    fn struct_and_trait() {
        assert_eq!(to_python("struct P(Stringable, Copyable):\n    var x: Int").unwrap(), "class P:\n    x: int");
        assert_eq!(to_python("trait T:\n    fn f(self): ...\nvar a = 1").unwrap(), "a = 1");
        assert_eq!(to_python("fn f[T: Copyable](x: T) -> T:").unwrap(), "def f(x: T) -> T:");
        assert_eq!(to_python("var p = Pair[Int, String](1, \"a\")").unwrap(), "p = Pair(1, \"a\")");
        assert!(to_python("@weird\nfn f(): pass").is_err());
        assert_eq!(to_python("@value\nstruct P:\n    var x: Int").unwrap(), "class P:\n    x: int");
    }
    #[test]
    fn ctors() {
        assert_eq!(to_python("var a = List[Int](1, 2, 3)").unwrap(), "a = [1, 2, 3]");
        assert_eq!(to_python("var d = Dict[String, Int]()").unwrap(), "d = {}");
        assert_eq!(to_python("var v = SIMD[DType.float64, 2](1.0, 2.0)").unwrap(), "v = simd(1.0, 2.0)");
    }
    #[test]
    fn strings_untouched() {
        assert_eq!(to_python("print(\"Int String\")").unwrap(), "print(\"Int String\")");
    }
}
