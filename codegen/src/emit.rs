//! Rust source emission: typed dictionary constants and PDO payload structs.

use std::collections::HashSet;
use std::fmt::Write as _;

use crate::model::{Eds, Object, ObjectKind, Sub, parse_nodeid_expr, parse_number};
use crate::names::{const_name, dedupe, item_name};
use crate::{Config, DefmtDerive, Error};

pub fn generate(eds: &Eds, config: &Config) -> Result<String, Error> {
    let mut out = String::new();
    let dict = format!("{}::dict", config.crate_path);

    emit_header(&mut out, eds, config);

    // Top-level namespaces: constants (VAR objects) and modules (the rest)
    let mut const_scope = HashSet::new();
    let mut module_scope = HashSet::new();

    for object in &eds.objects {
        match &object.kind {
            ObjectKind::Var(sub) => {
                let name = dedupe(
                    &mut const_scope,
                    const_name(&object.name).unwrap_or_else(|| format!("OBJ_{:04X}", object.index)),
                );
                emit_entry_const(&mut out, &dict, "", &name, object.index, sub);
            }
            ObjectKind::Array(subs) => {
                let module = dedupe(
                    &mut module_scope,
                    item_name(&object.name).unwrap_or_else(|| format!("obj_{:04x}", object.index)),
                );
                emit_array_module(&mut out, &dict, &module, object, subs);
            }
            ObjectKind::Record(subs) => {
                if subs.is_empty() {
                    continue; // unsupported object type placeholder
                }
                let module = dedupe(
                    &mut module_scope,
                    item_name(&object.name).unwrap_or_else(|| format!("obj_{:04x}", object.index)),
                );
                emit_record_module(&mut out, &dict, &module, object, subs);
            }
        }
    }

    if config.emit_pdos {
        emit_pdos(&mut out, eds, config)?;
    }

    Ok(out)
}

fn emit_header(out: &mut String, eds: &Eds, config: &Config) {
    let _ = writeln!(out, "// Generated code! Do not edit!");
    let _ = writeln!(
        out,
        "// Object dictionary of `{}` (vendor `{}`)",
        eds.device_info.product_name, eds.device_info.vendor_name
    );
    let _ = writeln!(
        out,
        "// Source: `{}` (FileVersion {}, FileRevision {})",
        config.eds_name, eds.file_info.file_version, eds.file_info.file_revision
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "use {}::dict::*;", config.crate_path);
    let _ = writeln!(out);
}

fn doc_line(sub: &Sub) -> String {
    let mut doc = format!("`{}` — {}", sub.name, sub.access.as_str());
    if !sub.default.is_empty() {
        let _ = write!(doc, ", default `{}`", sub.default);
    }
    match (&sub.low_limit, &sub.high_limit) {
        (Some(low), Some(high)) => {
            let _ = write!(doc, ", range `{low}`..=`{high}`");
        }
        (Some(low), None) => {
            let _ = write!(doc, ", min `{low}`");
        }
        (None, Some(high)) => {
            let _ = write!(doc, ", max `{high}`");
        }
        (None, None) => {}
    }
    doc
}

fn emit_entry_const(out: &mut String, dict: &str, indent: &str, name: &str, index: u16, sub: &Sub) {
    let _ = dict;
    let _ = writeln!(out, "{indent}/// {}", doc_line(sub));
    let _ = writeln!(
        out,
        "{indent}pub const {name}: SdoEntry<{ty}> = SdoEntry::new({index:#06X}, {sub:#04X}, \"{orig}\");",
        ty = sub.data_type.rust_type(),
        sub = sub.sub,
        orig = escape(&sub.name),
    );
}

fn emit_record_module(out: &mut String, dict: &str, module: &str, object: &Object, subs: &[Sub]) {
    let _ = writeln!(out, "/// `{}` ({:#06X}, RECORD)", escape(&object.name), object.index);
    let _ = writeln!(out, "pub mod {module} {{");
    let _ = writeln!(out, "    use {dict}::*;");
    let mut scope = HashSet::new();
    for sub in subs {
        let name = dedupe(
            &mut scope,
            const_name(&sub.name).unwrap_or_else(|| format!("SUB_{}", sub.sub)),
        );
        emit_entry_const(out, dict, "    ", &name, object.index, sub);
    }
    let _ = writeln!(out, "}}");
}

fn emit_array_module(out: &mut String, dict: &str, module: &str, object: &Object, subs: &[Sub]) {
    let elements: Vec<&Sub> = subs.iter().filter(|s| s.sub != 0).collect();
    let len = elements.iter().map(|s| s.sub).max().unwrap_or(0);
    let element_type = elements[0].data_type.rust_type();

    let _ = writeln!(
        out,
        "/// `{}` ({:#06X}, ARRAY): {len} × {element_type}",
        escape(&object.name),
        object.index
    );
    let _ = writeln!(out, "pub mod {module} {{");
    let _ = writeln!(out, "    use {dict}::*;");
    let _ = writeln!(
        out,
        "    pub const ENTRIES: SdoArray<{element_type}> = SdoArray::new({:#06X}, {len}, \"{}\");",
        object.index,
        escape(&object.name),
    );
    // Also emit the elements as named constants where the EDS names them
    // distinctively (e.g. `Nonce`/`Signature`), so call sites don't need
    // magic sub-indices.
    let mut scope = HashSet::new();
    scope.insert("ENTRIES".to_string());
    for sub in elements {
        let name = dedupe(
            &mut scope,
            const_name(&sub.name).unwrap_or_else(|| format!("SUB_{}", sub.sub)),
        );
        emit_entry_const(out, dict, "    ", &name, object.index, sub);
    }
    let _ = writeln!(out, "}}");
}

// ## --- PDO payload structs --- ##

struct PdoField {
    name: String,
    rust_type: &'static str,
    size: usize,
    doc: String,
}

/// One slot of a PDO payload: a decoded field or anonymous padding bytes
/// (CiA 301 dummy mappings, indices 0x0001–0x0007).
enum PdoPart {
    Field(PdoField),
    Padding(usize),
}

fn emit_pdos(out: &mut String, eds: &Eds, config: &Config) -> Result<(), Error> {
    // (comm base, mapping base, COB-ID function-code bases, struct prefix, is_tpdo)
    const KINDS: [(u16, u16, u16, &str, bool); 2] = [
        (0x1800, 0x1A00, 0x180, "Tpdo", true),
        (0x1400, 0x1600, 0x200, "Rpdo", false),
    ];

    for (comm_base, map_base, cob_base, prefix, is_tpdo) in KINDS {
        for comm in eds.objects.iter().filter(|o| (comm_base..comm_base + 0x200).contains(&o.index)) {
            let offset = comm.index - comm_base;
            let Some(mapping) = eds.find_object(map_base + offset) else {
                continue;
            };

            let parts = resolve_mapping_parts(eds, mapping)?;
            if !parts.iter().any(|p| matches!(p, PdoPart::Field(_))) {
                continue; // unmapped (or all-padding) PDO: nothing to generate
            }

            let num = pdo_number(comm, cob_base).unwrap_or(offset as u8 + 1);
            emit_pdo_struct(out, config, &format!("{prefix}{num}"), num, &parts, is_tpdo);
        }
    }
    Ok(())
}

/// PDO number (1-based) from the COB-ID default of communication sub 1.
fn pdo_number(comm: &Object, cob_base: u16) -> Option<u8> {
    let default = &comm.find_sub(1)?.default;
    let (value, _node_relative) = parse_nodeid_expr(default)?;
    // Mask the valid/RTR flag bits and the node id; keep the function code.
    let function_base = (value & 0x780) as u16;
    let num = (function_base.checked_sub(cob_base)? / 0x100) + 1;
    (1..=4).contains(&num).then_some(num as u8)
}

fn resolve_mapping_parts(eds: &Eds, mapping: &Object) -> Result<Vec<PdoPart>, Error> {
    let mut parts = Vec::new();
    let mut scope = HashSet::new();
    let mut total_bits = 0u32;

    for sub_index in 1..=mapping.subs().iter().map(|s| s.sub).max().unwrap_or(0) {
        let Some(sub) = mapping.find_sub(sub_index) else { break };
        let Some(raw) = parse_number(&sub.default).filter(|&v| v != 0) else {
            break; // first empty mapping slot terminates the list
        };

        let index = (raw >> 16) as u16;
        let target_sub = ((raw >> 8) & 0xFF) as u8;
        let bits = raw & 0xFF;
        let context = format!("mapping object {:#06X} sub {sub_index}: {raw:#010X}", mapping.index);

        if bits % 8 != 0 {
            return Err(Error(format!("{context}: {bits}-bit mapping is not byte-aligned (unsupported)")));
        }
        total_bits += bits;
        if total_bits > 64 {
            return Err(Error(format!("{context}: mappings exceed 64 bits")));
        }

        // Dummy mappings (basic data type indices) are padding, not data
        if (0x0001..=0x0007).contains(&index) {
            parts.push(PdoPart::Padding(bits as usize / 8));
            continue;
        }

        let target = eds
            .find_object(index)
            .and_then(|o| o.find_sub(target_sub))
            .ok_or_else(|| Error(format!("{context}: mapped object {index:#06X}:{target_sub:#04X} not in dictionary")))?;
        let size = target.data_type.scalar_size().ok_or_else(|| {
            Error(format!("{context}: mapped object `{}` is not a fixed-size scalar", target.name))
        })?;
        if size * 8 != bits as usize {
            return Err(Error(format!(
                "{context}: mapping length {bits} bits does not match `{}` ({} bits)",
                target.name,
                size * 8
            )));
        }

        let name = dedupe(
            &mut scope,
            item_name(&target.name).unwrap_or_else(|| format!("field_{sub_index}")),
        );
        parts.push(PdoPart::Field(PdoField {
            name,
            rust_type: target.data_type.rust_type(),
            size,
            doc: format!("`{}` ({index:#06X}:{target_sub:#04X})", target.name),
        }));
    }

    Ok(parts)
}

fn emit_pdo_struct(
    out: &mut String,
    config: &Config,
    struct_name: &str,
    num: u8,
    parts: &[PdoPart],
    is_tpdo: bool,
) {
    let crate_path = &config.crate_path;
    let total: usize = parts
        .iter()
        .map(|p| match p {
            PdoPart::Field(f) => f.size,
            PdoPart::Padding(bytes) => *bytes,
        })
        .sum();
    let direction = if is_tpdo { "transmitted by the node" } else { "received by the node" };

    let _ = writeln!(out);
    let _ = writeln!(out, "/// {} payload ({direction}, PDO {num}, {total} bytes)", struct_name);
    let _ = writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq)]");
    match &config.derive_defmt {
        DefmtDerive::Never => {}
        DefmtDerive::Feature(feature) => {
            let _ = writeln!(out, "#[cfg_attr(feature = \"{feature}\", derive(defmt::Format))]");
        }
    }
    let _ = writeln!(out, "pub struct {struct_name} {{");
    for part in parts {
        if let PdoPart::Field(field) = part {
            let _ = writeln!(out, "    /// {}", field.doc);
            let _ = writeln!(out, "    pub {}: {},", field.name, field.rust_type);
        }
    }
    let _ = writeln!(out, "}}");

    if is_tpdo {
        let _ = writeln!(out);
        let _ = writeln!(out, "impl {crate_path}::pdo::PdoPayload for {struct_name} {{");
        let _ = writeln!(out, "    fn decode(data: &[u8]) -> Option<Self> {{");
        let _ = writeln!(out, "        use {crate_path}::dict::SdoScalar;");
        if total == 1 {
            let _ = writeln!(out, "        if data.is_empty() {{");
        } else {
            let _ = writeln!(out, "        if data.len() < {total} {{");
        }
        let _ = writeln!(out, "            return None;");
        let _ = writeln!(out, "        }}");
        let _ = writeln!(out, "        Some(Self {{");
        let mut offset = 0usize;
        for part in parts {
            match part {
                PdoPart::Field(field) => {
                    let _ = writeln!(
                        out,
                        "            {}: <{} as SdoScalar>::from_le_bytes(&data[{}..{}]),",
                        field.name,
                        field.rust_type,
                        offset,
                        offset + field.size
                    );
                    offset += field.size;
                }
                PdoPart::Padding(bytes) => offset += bytes,
            }
        }
        let _ = writeln!(out, "        }})");
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "}}");
    } else {
        let _ = writeln!(out);
        let _ = writeln!(out, "impl {struct_name} {{");
        let _ = writeln!(
            out,
            "    /// Encode for transmission via `NodeClient::send_rpdo({num}, ..)`."
        );
        let _ = writeln!(out, "    pub fn to_bytes(self) -> ([u8; 8], usize) {{");
        let _ = writeln!(out, "        use {crate_path}::dict::SdoScalar;");
        let _ = writeln!(out, "        let mut data = [0u8; 8];");
        let mut offset = 0usize;
        for part in parts {
            match part {
                PdoPart::Field(field) => {
                    let _ = writeln!(
                        out,
                        "        SdoScalar::write_le_bytes(self.{}, &mut data[{}..{}]);",
                        field.name,
                        offset,
                        offset + field.size
                    );
                    offset += field.size;
                }
                // Padding bytes stay zero
                PdoPart::Padding(bytes) => offset += bytes,
            }
        }
        let _ = writeln!(out, "        (data, {total})");
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "}}");
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
