//! Tolerant INI reading for vendor EDS files.
//!
//! Vendor files are messy: latin-1-ish encodings, commented-out lines
//! (`;StorageLocation=ROM`), stray whitespace, inconsistent key casing.
//! This reader is deliberately hand-rolled so tolerance stays under our
//! control instead of an INI crate's.

use std::path::Path;

use crate::Error;

/// One `[section]` with its key/value pairs in file order. Keys are stored
/// lowercased (EDS keys are case-insensitive), values verbatim.
#[derive(Debug)]
pub struct Section {
    pub name: String,
    entries: Vec<(String, String)>,
}

impl Section {
    pub fn get(&self, key: &str) -> Option<&str> {
        let key = key.to_ascii_lowercase();
        self.entries
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Like [`Self::get`], but empty values read as absent.
    pub fn get_nonempty(&self, key: &str) -> Option<&str> {
        self.get(key).filter(|v| !v.is_empty())
    }
}

/// A parsed INI document; sections in file order.
#[derive(Debug)]
pub struct Ini {
    pub sections: Vec<Section>,
}

impl Ini {
    pub fn get(&self, name: &str) -> Option<&Section> {
        self.sections
            .iter()
            .find(|s| s.name.eq_ignore_ascii_case(name))
    }
}

/// Parse INI text. Unrecognizable lines are skipped (collected into
/// `diagnostics` by the caller via the return value's second element).
pub fn parse_ini(src: &str) -> (Ini, Vec<String>) {
    let mut sections: Vec<Section> = Vec::new();
    let mut diagnostics = Vec::new();

    for (line_no, raw) in src.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }

        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            sections.push(Section {
                name: name.trim().to_string(),
                entries: Vec::new(),
            });
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            match sections.last_mut() {
                Some(section) => section
                    .entries
                    .push((key.trim().to_ascii_lowercase(), value.trim().to_string())),
                None => diagnostics.push(format!("line {}: key/value before any section", line_no + 1)),
            }
            continue;
        }

        diagnostics.push(format!("line {}: unrecognized line skipped: {line}", line_no + 1));
    }

    (Ini { sections }, diagnostics)
}

/// Read an EDS file, decoding as UTF-8 or, failing that, latin-1 (each byte
/// maps to the same code point). Replaces external `iconv` preprocessing.
pub fn read_eds_file(path: impl AsRef<Path>) -> Result<String, Error> {
    let path = path.as_ref();
    let bytes = std::fs::read(path)
        .map_err(|e| Error(format!("failed to read {}: {e}", path.display())))?;
    Ok(decode_bytes(&bytes))
}

fn decode_bytes(bytes: &[u8]) -> String {
    match core::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        // Latin-1: every byte is the identical code point
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_and_case_insensitive_keys() {
        let (ini, diags) = parse_ini(
            "[FileInfo]\nFileName=x.eds\n\n[1018sub1]\nParameterName=Vendor-ID\nDataType=0x0007\n",
        );
        assert!(diags.is_empty());
        assert_eq!(ini.get("fileinfo").unwrap().get("filename"), Some("x.eds"));
        let sub = ini.get("1018SUB1").unwrap();
        assert_eq!(sub.get("PARAMETERNAME"), Some("Vendor-ID"));
        assert_eq!(sub.get("datatype"), Some("0x0007"));
    }

    #[test]
    fn skips_comments_and_junk() {
        let (ini, diags) = parse_ini("[A]\n;StorageLocation=ROM\nk=v\ngarbage line\n");
        assert_eq!(ini.get("A").unwrap().get("k"), Some("v"));
        assert_eq!(ini.get("A").unwrap().get("storagelocation"), None);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn empty_values_read_as_absent_via_get_nonempty() {
        let (ini, _) = parse_ini("[A]\nDefaultValue=\n");
        let a = ini.get("A").unwrap();
        assert_eq!(a.get("defaultvalue"), Some(""));
        assert_eq!(a.get_nonempty("defaultvalue"), None);
    }

    #[test]
    fn latin1_fallback_decoding() {
        // "°C" in latin-1: 0xB0 0x43 — invalid UTF-8
        let decoded = decode_bytes(&[b'[', b'A', b']', b'\n', b'k', b'=', 0xB0, b'C', b'\n']);
        let (ini, _) = parse_ini(&decoded);
        assert_eq!(ini.get("A").unwrap().get("k"), Some("°C"));
    }
}
