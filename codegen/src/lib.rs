//! Generate typed [canopen-async](../canopen_async/index.html) object
//! dictionaries and PDO payload structs from CANopen EDS files.
//!
//! Intended for use from a `build.rs`. The simplest flow generates into
//! `OUT_DIR` under a label and includes it via canopen-async's
//! `include_dictionary!` macro:
//!
//! ```no_run
//! // build.rs
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! canopen_async_codegen::build_dictionary("ENYRING", "eds/bat_enyring.eds")?;
//! # Ok(())
//! # }
//! ```
//!
//! ```ignore
//! // main.rs — the wrapper's #[allow] covers lints for unused entries
//! #[allow(dead_code)]
//! mod dict {
//!     canopen_async::include_dictionary!(ENYRING);
//! }
//! ```
//!
//! Parser diagnostics (skipped lines, count mismatches, suspicious default
//! values) are printed as `cargo:warning` lines. For full control over
//! output location and [`Config`] options, use [`read_eds_file`] +
//! [`Config::generate`] directly and `include!` the file yourself.

use std::path::{Path, PathBuf};

mod emit;
mod model;
mod names;
mod parse;

pub use parse::read_eds_file;

/// Generate a dictionary from a build script, for inclusion via
/// canopen-async's `include_dictionary!` macro.
///
/// Writes the generated code to `OUT_DIR`, exports its path as the
/// `CANOPEN_DICTIONARY_<NAME>` environment variable (consumed by the macro),
/// emits `cargo:rerun-if-changed` for the EDS file, and surfaces parser
/// diagnostics as `cargo:warning` lines.
pub fn build_dictionary(name: &str, eds_path: impl AsRef<Path>) -> Result<(), Error> {
    build_dictionary_with(name, eds_path, |config| config)
}

/// Like [`build_dictionary`], with a hook to adjust the [`Config`]
/// (e.g. `|c| c.derive_defmt(DefmtDerive::Feature("defmt-print".into()))`).
pub fn build_dictionary_with(
    name: &str,
    eds_path: impl AsRef<Path>,
    configure: impl FnOnce(Config) -> Config,
) -> Result<(), Error> {
    let out_dir = std::env::var_os("OUT_DIR").ok_or_else(|| {
        Error("OUT_DIR is not set; build_dictionary must run from a build script".to_string())
    })?;
    let eds_path = eds_path.as_ref();

    let (file, diagnostics) = generate_dictionary_file(name, eds_path, Path::new(&out_dir), configure)?;

    println!("cargo:rerun-if-changed={}", eds_path.display());
    println!("cargo:rustc-env=CANOPEN_DICTIONARY_{name}={}", file.display());
    for diagnostic in diagnostics {
        println!("cargo:warning={}: {diagnostic}", eds_path.display());
    }
    Ok(())
}

/// Build-script-independent core of [`build_dictionary`]: generate into
/// `out_dir/canopen_dictionary_<name>.rs` and return the path plus parser
/// diagnostics.
pub fn generate_dictionary_file(
    name: &str,
    eds_path: &Path,
    out_dir: &Path,
    configure: impl FnOnce(Config) -> Config,
) -> Result<(PathBuf, Vec<String>), Error> {
    if name.is_empty()
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        || name.starts_with(|c: char| c.is_ascii_digit())
    {
        return Err(Error(format!(
            "dictionary name `{name}` must be a valid identifier (used in the CANOPEN_DICTIONARY_<NAME> env var)"
        )));
    }

    let source = read_eds_file(eds_path)?;
    let eds_name = eds_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| eds_path.display().to_string());

    let config = configure(Config::new(&eds_name, &source));
    let (code, diagnostics) = config.generate_with_diagnostics()?;

    let file = out_dir.join(format!("canopen_dictionary_{name}.rs"));
    std::fs::write(&file, code)
        .map_err(|e| Error(format!("failed to write {}: {e}", file.display())))?;
    Ok((file, diagnostics))
}

/// Generation failure: unparsable or unsupported EDS content. The message
/// names the object/sub-entry concerned.
#[derive(Debug)]
pub struct Error(pub(crate) String);

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// Whether generated PDO structs derive `defmt::Format`.
#[derive(Debug, Clone, Default)]
pub enum DefmtDerive {
    /// No defmt derive.
    #[default]
    Never,
    /// Derive behind `#[cfg_attr(feature = "...")]` with the given feature
    /// name of the *consuming* crate (e.g. `defmt-print` in cmcu-next).
    Feature(String),
}

/// Code generation configuration.
#[derive(Debug)]
pub struct Config<'a> {
    eds_name: &'a str,
    source: &'a str,
    emit_pdos: bool,
    crate_path: String,
    derive_defmt: DefmtDerive,
}

impl<'a> Config<'a> {
    /// `eds_name` appears in the generated header; `source` is the EDS text
    /// (see [`read_eds_file`] for encoding-tolerant loading).
    pub fn new(eds_name: &'a str, source: &'a str) -> Self {
        Self {
            eds_name,
            source,
            emit_pdos: true,
            crate_path: "::canopen_async".to_string(),
            derive_defmt: DefmtDerive::Never,
        }
    }

    /// Generate `PdoPayload` structs from the PDO mapping defaults
    /// (default: on; PDOs without mappings are skipped either way).
    pub fn emit_pdos(mut self, emit: bool) -> Self {
        self.emit_pdos = emit;
        self
    }

    /// Path of the canopen-async crate in the consuming code
    /// (default `::canopen_async`).
    pub fn crate_path(mut self, path: &str) -> Self {
        self.crate_path = path.to_string();
        self
    }

    /// Derive `defmt::Format` on generated PDO structs.
    pub fn derive_defmt(mut self, derive: DefmtDerive) -> Self {
        self.derive_defmt = derive;
        self
    }

    /// Generate the Rust source. Non-fatal EDS oddities are reported via
    /// [`generate_with_diagnostics`](Self::generate_with_diagnostics).
    pub fn generate(&self) -> Result<String, Error> {
        self.generate_with_diagnostics().map(|(code, _)| code)
    }

    /// Like [`generate`](Self::generate), also returning parser diagnostics
    /// (skipped lines, membership-list mismatches, …) worth surfacing as
    /// build warnings.
    pub fn generate_with_diagnostics(&self) -> Result<(String, Vec<String>), Error> {
        let eds = model::Eds::parse(self.source)?;
        let code = emit::generate(&eds, self)?;
        Ok((code, eds.diagnostics))
    }
}
