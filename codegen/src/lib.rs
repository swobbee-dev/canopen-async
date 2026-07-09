//! Generate typed [canopen-async](../canopen_async/index.html) object
//! dictionaries and PDO payload structs from CANopen EDS files.
//!
//! Intended for use from a `build.rs`, mirroring the dbc-codegen pattern:
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let source = canopen_async_codegen::read_eds_file("eds/bat_enyring.eds")?;
//! let code = canopen_async_codegen::Config::new("bat_enyring.eds", &source).generate()?;
//! std::fs::write("out/enyring_dict.rs", code)?;
//! # Ok(())
//! # }
//! ```
//!
//! Include the output in a wrapping module (the wrapper's `#[allow]` covers
//! lints for unused entries):
//!
//! ```ignore
//! #[allow(dead_code)]
//! mod dict {
//!     include!(concat!(env!("OUT_DIR"), "/enyring_dict.rs"));
//! }
//! ```

mod emit;
mod model;
mod names;
mod parse;

pub use parse::read_eds_file;

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
