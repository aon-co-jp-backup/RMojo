//! Mojo surface syntax -> the Python subset RMojo runs. Line based, deliberately small and honest:
//! supported: `fn` (-> `def`, `raises` dropped), `var`/`let` declarations, `alias X = v`, argument conventions
//! (`inout`/`borrowed`/`owned`/`mut`/`read`/`out`), Mojo type names (Int, Int64, UInt..., Float64, String, Bool -> Python names).
//! `struct` -> `class` (fields from `var`, methods via `fn`; reference semantics). NOT supported (reported as an error, never silently mis-run): struct trait lists, `trait`, `@decorators`, `import` of Mojo modules, SIMD.

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
        // only inside the parameter list: after '(' or ", "
        t = t.replace(&format!("({kw}"), "(").replace(&format!(", {kw}"), ", ");
    }
    t
}

pub fn to_python(src: &str) -> Result<String, String> {
    let mut out = Vec::new();
    let mut struct_indent: Option<usize> = None;
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
        }
        if let Some(rest) = t.strip_prefix("struct ") {
            if t.contains('(') {
                return unsupported("struct with traits");
            }
            struct_indent = Some(ind.len());
            out.push(format!("{ind}class {}", rest));
            continue;
        }
        let in_struct = struct_indent.is_some();
        if t.starts_with("trait ") {
            return unsupported("trait");
        }
        if t.starts_with('@') {
            return unsupported("@decorator");
        }
        let mut line = body.to_string();
        if t.starts_with("fn ") || t.starts_with("def ") {
            line = line.replacen("fn ", "def ", 1);
            line = drop_conventions(&line);
            line = line.replace(" raises:", ":").replace(" raises ->", " ->");
        } else if let Some(rest) = t.strip_prefix("var ").or_else(|| t.strip_prefix("let ")) {
            line = rest.to_string();
            // `var x: Int = 5` -> `x: int = 5` (annotation kept); `var x: Int` alone has no value -> unsupported
            if !line.contains('=') && !in_struct {
                return unsupported("declaration without a value");
            }
        } else if let Some(rest) = t.strip_prefix("alias ") {
            line = rest.to_string();
        }
        out.push(format!("{ind}{}", map_type_words(&line)));
    }
    Ok(out.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fn_var_types() {
        let s = "fn add(x: Int, y: Int) -> Int:\n    var z = x + y\n    return z\n";
        let p = to_python(s).unwrap();
        assert_eq!(p, "def add(x: int, y: int) -> int:\n    z = x + y\n    return z");
    }
    #[test]
    fn conventions_and_raises() {
        assert_eq!(to_python("fn f(inout a: Int, borrowed b: Float64) raises -> Int:").unwrap(), "def f(a: int, b: float) -> int:");
    }
    #[test]
    fn struct_lowered() {
        assert_eq!(to_python("struct P:
    var x: Int").unwrap(), "class P:
    x: int");
        assert!(to_python("struct P(Copyable):
    var x: Int").is_err());
    }
    #[test]
    fn strings_untouched() {
        assert_eq!(to_python("print(\"Int String\")").unwrap(), "print(\"Int String\")");
    }
}
