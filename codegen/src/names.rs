//! Rust identifier generation from EDS parameter names.

use std::collections::HashSet;

/// SCREAMING_SNAKE_CASE for constants; `None` if nothing survives sanitizing.
pub fn const_name(raw: &str) -> Option<String> {
    let s = sanitize(raw)?.to_ascii_uppercase();
    Some(s)
}

/// snake_case for modules and struct fields; `None` if nothing survives.
pub fn item_name(raw: &str) -> Option<String> {
    let s = sanitize(raw)?.to_ascii_lowercase();
    // Modules/fields can collide with keywords; constants cannot (uppercase).
    Some(if is_keyword(&s) { format!("{s}_") } else { s })
}

/// Collapse a free-form name to `[A-Za-z0-9_]+` with single underscores,
/// splitting CamelCase words (`SlaveAssignment` → `Slave_Assignment`,
/// `NMTStartup` → `NMT_Startup`).
fn sanitize(raw: &str) -> Option<String> {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut last_was_sep = true; // suppress leading separators
    for (i, &ch) in chars.iter().enumerate() {
        if ch.is_ascii_alphanumeric() {
            let camel_boundary = ch.is_ascii_uppercase()
                && i > 0
                && (chars[i - 1].is_ascii_lowercase()
                    || chars[i - 1].is_ascii_digit()
                    || (chars[i - 1].is_ascii_uppercase()
                        && chars.get(i + 1).is_some_and(|c| c.is_ascii_lowercase())));
            if camel_boundary && !last_was_sep {
                out.push('_');
            }
            out.push(ch);
            last_was_sep = false;
        } else if !last_was_sep {
            out.push('_');
            last_was_sep = true;
        }
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        return None;
    }
    if out.chars().next().unwrap().is_ascii_digit() {
        out.insert(0, '_');
    }
    Some(out)
}

/// Reserve `base` in `scope`, appending `_2`, `_3`… on collision.
pub fn dedupe(scope: &mut HashSet<String>, base: String) -> String {
    if scope.insert(base.clone()) {
        return base;
    }
    for n in 2.. {
        let candidate = format!("{base}_{n}");
        if scope.insert(candidate.clone()) {
            return candidate;
        }
    }
    unreachable!()
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn" | "else"
            | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop"
            | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return" | "self" | "static"
            | "struct" | "super" | "trait" | "true" | "type" | "unsafe" | "use" | "where"
            | "while"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_typical_eds_names() {
        assert_eq!(const_name("State of charge").as_deref(), Some("STATE_OF_CHARGE"));
        assert_eq!(const_name("Vendor-ID").as_deref(), Some("VENDOR_ID"));
        assert_eq!(const_name("COB-ID used by RPDO").as_deref(), Some("COB_ID_USED_BY_RPDO"));
        assert_eq!(item_name("Cells voltage").as_deref(), Some("cells_voltage"));
        assert_eq!(item_name("Bike 26").as_deref(), Some("bike_26"));
    }

    #[test]
    fn splits_camel_case() {
        assert_eq!(item_name("SlaveAssignment").as_deref(), Some("slave_assignment"));
        assert_eq!(item_name("NMTStartup").as_deref(), Some("nmt_startup"));
        assert_eq!(item_name("RequestNMT").as_deref(), Some("request_nmt"));
        assert_eq!(const_name("FW Update").as_deref(), Some("FW_UPDATE"));
    }

    #[test]
    fn handles_digits_keywords_and_empties() {
        assert_eq!(const_name("4wd mode").as_deref(), Some("_4WD_MODE"));
        assert_eq!(item_name("Type").as_deref(), Some("type_"));
        assert_eq!(const_name(""), None);
        assert_eq!(const_name("---"), None);
    }

    #[test]
    fn dedupe_appends_counters() {
        let mut scope = HashSet::new();
        assert_eq!(dedupe(&mut scope, "NONCE".into()), "NONCE");
        assert_eq!(dedupe(&mut scope, "NONCE".into()), "NONCE_2");
        assert_eq!(dedupe(&mut scope, "NONCE".into()), "NONCE_3");
    }
}
