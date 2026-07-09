//! Object-dictionary model built from a parsed EDS.

use crate::Error;
use crate::parse::{Ini, Section, parse_ini};

/// CiA 301 basic data types supported by the generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Boolean,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    VisibleString,
    OctetString,
    UnicodeString,
    Domain,
}

impl DataType {
    pub fn from_code(code: u16) -> Option<Self> {
        Some(match code {
            0x0001 => DataType::Boolean,
            0x0002 => DataType::I8,
            0x0003 => DataType::I16,
            0x0004 => DataType::I32,
            0x0005 => DataType::U8,
            0x0006 => DataType::U16,
            0x0007 => DataType::U32,
            0x0008 => DataType::F32,
            0x0009 => DataType::VisibleString,
            0x000A => DataType::OctetString,
            0x000B => DataType::UnicodeString,
            0x000F => DataType::Domain,
            0x0011 => DataType::F64,
            0x0015 => DataType::I64,
            0x001B => DataType::U64,
            _ => return None,
        })
    }

    /// The Rust type parameter of the generated `SdoEntry`.
    pub fn rust_type(&self) -> &'static str {
        match self {
            DataType::Boolean => "bool",
            DataType::I8 => "i8",
            DataType::I16 => "i16",
            DataType::I32 => "i32",
            DataType::I64 => "i64",
            DataType::U8 => "u8",
            DataType::U16 => "u16",
            DataType::U32 => "u32",
            DataType::U64 => "u64",
            DataType::F32 => "f32",
            DataType::F64 => "f64",
            DataType::VisibleString => "VisibleString",
            DataType::OctetString => "OctetString",
            DataType::UnicodeString => "OctetString", // no dedicated marker; raw bytes
            DataType::Domain => "Domain",
        }
    }

    /// Size in bytes for fixed-size scalars; `None` for strings/domain.
    pub fn scalar_size(&self) -> Option<usize> {
        Some(match self {
            DataType::Boolean | DataType::I8 | DataType::U8 => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::F32 => 4,
            DataType::I64 | DataType::U64 | DataType::F64 => 8,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    /// rwr: read/write on process input (TPDO-mappable)
    ReadWriteRead,
    /// rww: read/write on process output (RPDO-mappable)
    ReadWriteWrite,
    Const,
}

impl AccessType {
    fn from_str(s: &str) -> Option<Self> {
        Some(match s.to_ascii_lowercase().as_str() {
            "ro" => AccessType::ReadOnly,
            "wo" => AccessType::WriteOnly,
            "rw" => AccessType::ReadWrite,
            "rwr" => AccessType::ReadWriteRead,
            "rww" => AccessType::ReadWriteWrite,
            "const" => AccessType::Const,
            _ => return None,
        })
    }

    /// The EDS spelling, used in generated doc comments.
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessType::ReadOnly => "ro",
            AccessType::WriteOnly => "wo",
            AccessType::ReadWrite => "rw",
            AccessType::ReadWriteRead => "rwr",
            AccessType::ReadWriteWrite => "rww",
            AccessType::Const => "const",
        }
    }
}

/// One sub-entry (or the single variable of a VAR object).
#[derive(Debug, Clone)]
pub struct Sub {
    pub sub: u8,
    pub name: String,
    pub data_type: DataType,
    pub access: AccessType,
    pub default: String,
    pub low_limit: Option<String>,
    pub high_limit: Option<String>,
}

#[derive(Debug)]
pub enum ObjectKind {
    Var(Sub),
    /// Homogeneous array: sub 0 is the element count, elements share a type.
    Array(Vec<Sub>),
    Record(Vec<Sub>),
}

#[derive(Debug)]
pub struct Object {
    pub index: u16,
    pub name: String,
    pub kind: ObjectKind,
}

impl Object {
    /// All subs, regardless of kind (a VAR yields its single entry).
    pub fn subs(&self) -> &[Sub] {
        match &self.kind {
            ObjectKind::Var(sub) => core::slice::from_ref(sub),
            ObjectKind::Array(subs) | ObjectKind::Record(subs) => subs,
        }
    }

    pub fn find_sub(&self, sub: u8) -> Option<&Sub> {
        self.subs().iter().find(|s| s.sub == sub)
    }
}

#[derive(Debug, Default)]
pub struct DeviceInfo {
    pub vendor_name: String,
    pub product_name: String,
}

#[derive(Debug, Default)]
pub struct FileInfo {
    pub file_name: String,
    pub file_version: String,
    pub file_revision: String,
}

/// A parsed EDS: device metadata plus the object dictionary, ordered by index.
#[derive(Debug)]
pub struct Eds {
    pub file_info: FileInfo,
    pub device_info: DeviceInfo,
    pub objects: Vec<Object>,
    /// Non-fatal oddities found while parsing.
    pub diagnostics: Vec<String>,
}

impl Eds {
    pub fn parse(source: &str) -> Result<Self, Error> {
        let (ini, mut diagnostics) = parse_ini(source);

        let mut file_info = FileInfo::default();
        if let Some(s) = ini.get("FileInfo") {
            file_info.file_name = s.get("FileName").unwrap_or_default().to_string();
            file_info.file_version = s.get("FileVersion").unwrap_or_default().to_string();
            file_info.file_revision = s.get("FileRevision").unwrap_or_default().to_string();
        }
        let mut device_info = DeviceInfo::default();
        if let Some(s) = ini.get("DeviceInfo") {
            device_info.vendor_name = s.get("VendorName").unwrap_or_default().to_string();
            device_info.product_name = s.get("ProductName").unwrap_or_default().to_string();
        }

        // Classify sections into objects and their sub-entries.
        let mut objects = Vec::new();
        for section in &ini.sections {
            if let Some(index) = object_section_index(&section.name) {
                objects.push(build_object(index, section, &ini, &mut diagnostics)?);
            }
        }
        objects.sort_by_key(|o| o.index);

        cross_check_membership_lists(&ini, &objects, &mut diagnostics);

        Ok(Eds {
            file_info,
            device_info,
            objects,
            diagnostics,
        })
    }

    pub fn find_object(&self, index: u16) -> Option<&Object> {
        self.objects.iter().find(|o| o.index == index)
    }
}

/// `[1A00]` → object index; sub-entry and meta sections yield `None`.
fn object_section_index(name: &str) -> Option<u16> {
    (name.len() == 4).then(|| u16::from_str_radix(name, 16).ok()).flatten()
}

fn build_object(
    index: u16,
    section: &Section,
    ini: &Ini,
    diagnostics: &mut Vec<String>,
) -> Result<Object, Error> {
    let name = section.get("ParameterName").unwrap_or_default().to_string();

    if section.get_nonempty("CompactSubObj").is_some() {
        return Err(Error(format!(
            "object {index:#06X} `{name}`: CompactSubObj is not supported (expand the sub-entries in the EDS)"
        )));
    }

    // ObjectType defaults to VAR (0x7) per DS306
    let object_type = parse_number(section.get_nonempty("ObjectType").unwrap_or("0x7"))
        .ok_or_else(|| Error(format!("object {index:#06X}: unparsable ObjectType")))?;

    match object_type {
        0x7 => {
            let sub = build_sub(index, 0, name.clone(), section, diagnostics)?;
            Ok(Object {
                index,
                name,
                kind: ObjectKind::Var(sub),
            })
        }
        0x8 | 0x9 => {
            let mut subs = Vec::new();
            // Discover subs by section presence; SubNumber is cross-checked only.
            for sub_section in &ini.sections {
                if let Some(sub_index) = sub_section_index(&sub_section.name, index) {
                    let sub_name = sub_section.get("ParameterName").unwrap_or_default().to_string();
                    subs.push(build_sub(index, sub_index, sub_name, sub_section, diagnostics)?);
                }
            }
            subs.sort_by_key(|s| s.sub);

            if let Some(declared) = section.get_nonempty("SubNumber").and_then(parse_number)
                && declared as usize != subs.len()
            {
                diagnostics.push(format!(
                    "object {index:#06X} `{name}`: SubNumber={declared} but {} sub-entry sections found",
                    subs.len()
                ));
            }

            let kind = if object_type == 0x8 {
                // EDS arrays are homogeneous; tolerate violations as RECORD
                let element_types: Vec<_> =
                    subs.iter().filter(|s| s.sub != 0).map(|s| s.data_type).collect();
                if element_types.windows(2).all(|w| w[0] == w[1]) && !element_types.is_empty() {
                    // Sub 0 usually declares the element count. A nonzero
                    // declared count disagreeing with the highest element
                    // sub-index usually means a truncated file. Zero is
                    // exempt: some arrays report a *dynamic* count there
                    // (e.g. 0x1003, number of active errors).
                    let max_sub = subs.iter().map(|s| s.sub).max().unwrap_or(0);
                    if let Some(declared) = subs
                        .iter()
                        .find(|s| s.sub == 0)
                        .and_then(|s| parse_number(&s.default))
                        .filter(|&count| count != 0)
                        && declared != max_sub as u32
                    {
                        diagnostics.push(format!(
                            "object {index:#06X} `{name}`: sub 0 declares {declared} elements but the highest element sub-index is {max_sub}"
                        ));
                    }
                    ObjectKind::Array(subs)
                } else {
                    diagnostics.push(format!(
                        "object {index:#06X} `{name}`: ARRAY with heterogeneous or missing element types, emitting as RECORD"
                    ));
                    ObjectKind::Record(subs)
                }
            } else {
                ObjectKind::Record(subs)
            };

            Ok(Object { index, name, kind })
        }
        other => {
            diagnostics.push(format!(
                "object {index:#06X} `{name}`: unsupported ObjectType {other:#X} skipped"
            ));
            // Represent as an empty record so membership checks still work
            Ok(Object {
                index,
                name,
                kind: ObjectKind::Record(Vec::new()),
            })
        }
    }
}

/// `[1A00sub2]`-style section for the given object → sub-index.
fn sub_section_index(name: &str, object_index: u16) -> Option<u8> {
    let rest = name.get(..4).and_then(|prefix| {
        (u16::from_str_radix(prefix, 16).ok()? == object_index).then(|| &name[4..])
    })?;
    let sub_hex = rest.strip_prefix("sub").or_else(|| rest.strip_prefix("Sub"))?;
    u8::from_str_radix(sub_hex, 16).ok()
}

fn build_sub(
    index: u16,
    sub: u8,
    name: String,
    section: &Section,
    diagnostics: &mut Vec<String>,
) -> Result<Sub, Error> {
    let dt_raw = section.get_nonempty("DataType").ok_or_else(|| {
        Error(format!("object {index:#06X} sub {sub:#04X} `{name}`: missing DataType"))
    })?;
    let dt_code = parse_number(dt_raw)
        .ok_or_else(|| Error(format!("object {index:#06X} sub {sub:#04X}: unparsable DataType `{dt_raw}`")))?;
    let data_type = DataType::from_code(dt_code as u16).ok_or_else(|| {
        Error(format!(
            "object {index:#06X} sub {sub:#04X} `{name}`: unsupported DataType {dt_code:#06X}"
        ))
    })?;

    let access = section
        .get_nonempty("AccessType")
        .and_then(AccessType::from_str)
        .unwrap_or(AccessType::ReadWrite);

    let entry = Sub {
        sub,
        name,
        data_type,
        access,
        default: section.get("DefaultValue").unwrap_or_default().to_string(),
        low_limit: section.get_nonempty("LowLimit").map(str::to_string),
        high_limit: section.get_nonempty("HighLimit").map(str::to_string),
    };
    validate_default(index, &entry, diagnostics);
    Ok(entry)
}

/// Diagnose defaults that do not parse as (or exceed the range of) the
/// declared scalar type. Strings/domain and `$NODEID+…` expressions are
/// exempt; a broken default usually means a broken vendor file.
fn validate_default(index: u16, sub: &Sub, diagnostics: &mut Vec<String>) {
    let value = sub.default.trim();
    if value.is_empty() || value.starts_with("$NODEID") || value.starts_with("$NodeID") {
        return;
    }
    let Some(size) = sub.data_type.scalar_size() else {
        return; // strings/domain carry free-form defaults
    };

    let context = || {
        format!(
            "object {index:#06X} sub {:#04X} `{}`: DefaultValue `{value}`",
            sub.sub, sub.name
        )
    };

    match sub.data_type {
        DataType::F32 | DataType::F64 => {
            if value.parse::<f64>().is_err() {
                diagnostics.push(format!("{} does not parse as a float", context()));
            }
        }
        DataType::Boolean => {
            if !matches!(value, "0" | "1" | "true" | "false") {
                diagnostics.push(format!("{} is not a boolean", context()));
            }
        }
        _ => {
            let parsed: Option<i128> = if let Some(hex) =
                value.strip_prefix("0x").or_else(|| value.strip_prefix("0X"))
            {
                i128::from_str_radix(hex, 16).ok()
            } else {
                value.parse().ok()
            };
            let Some(parsed) = parsed else {
                diagnostics.push(format!("{} does not parse as an integer", context()));
                return;
            };
            // Hex defaults for signed types conventionally carry the two's
            // complement bit pattern, so the upper bound is the unsigned
            // maximum of the type's width either way.
            let max = if size == 8 { u64::MAX as i128 } else { (1i128 << (size * 8)) - 1 };
            let min = match sub.data_type {
                DataType::I8 => i8::MIN as i128,
                DataType::I16 => i16::MIN as i128,
                DataType::I32 => i32::MIN as i128,
                DataType::I64 => i64::MIN as i128,
                _ => 0,
            };
            if parsed < min || parsed > max {
                diagnostics.push(format!(
                    "{} is out of range for {}",
                    context(),
                    sub.data_type.rust_type()
                ));
            }
        }
    }
}

fn cross_check_membership_lists(ini: &Ini, objects: &[Object], diagnostics: &mut Vec<String>) {
    for list in ["MandatoryObjects", "OptionalObjects", "ManufacturerObjects"] {
        let Some(section) = ini.get(list) else { continue };
        let declared = section.get_nonempty("SupportedObjects").and_then(parse_number);
        let mut listed = 0u32;
        let mut n = 1;
        while let Some(value) = section.get_nonempty(&n.to_string()) {
            listed += 1;
            if let Some(index) = parse_number(value)
                && !objects.iter().any(|o| o.index == index as u16)
            {
                diagnostics.push(format!("{list} lists {index:#06X} but no such object section exists"));
            }
            n += 1;
        }
        if let Some(declared) = declared
            && declared != listed
        {
            diagnostics.push(format!("{list}: SupportedObjects={declared} but {listed} entries listed"));
        }
    }
}

/// Parse `0x…` hex or decimal.
pub fn parse_number(s: &str) -> Option<u32> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse().ok()
    }
}

/// Parse a DefaultValue that may be `$NODEID+0x…` (or a plain number);
/// returns (base value, node-id-relative).
pub fn parse_nodeid_expr(s: &str) -> Option<(u32, bool)> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("$NODEID+").or_else(|| s.strip_prefix("$NodeID+")) {
        Some((parse_number(rest)?, true))
    } else {
        Some((parse_number(s)?, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = "\
[1000]
ParameterName=Device type
ObjectType=0x7
DataType=0x0007
AccessType=ro
DefaultValue=0x00000000

[4242]
ParameterName=Cells voltage
ObjectType=0x8
SubNumber=0x3

[4242sub0]
ParameterName=Highest sub-index supported
DataType=0x0005
AccessType=ro
DefaultValue=2

[4242sub1]
ParameterName=Sub Object 1
DataType=0x0006
AccessType=ro

[4242sub2]
ParameterName=Sub Object 2
DataType=0x0006
AccessType=ro
";

    #[test]
    fn parses_var_and_array() {
        let eds = Eds::parse(MINIMAL).unwrap();
        assert_eq!(eds.objects.len(), 2);

        let var = eds.find_object(0x1000).unwrap();
        assert!(matches!(&var.kind, ObjectKind::Var(s)
            if s.data_type == DataType::U32 && s.access == AccessType::ReadOnly));

        let array = eds.find_object(0x4242).unwrap();
        let ObjectKind::Array(subs) = &array.kind else {
            panic!("expected ARRAY")
        };
        assert_eq!(subs.len(), 3);
        assert_eq!(subs[1].data_type, DataType::U16);
    }

    #[test]
    fn hex_sub_sections_and_missing_object_type_default_to_var() {
        let eds = Eds::parse(
            "[4211]\nParameterName=A\nObjectType=0x8\n\n[4211sub1A]\nParameterName=Bike 26\nDataType=0x000F\nAccessType=rw\n\n[2000]\nParameterName=NoType\nDataType=0x0005\nAccessType=ro\n",
        )
        .unwrap();
        let arr = eds.find_object(0x4211).unwrap();
        assert_eq!(arr.subs()[0].sub, 0x1A);
        assert_eq!(arr.subs()[0].data_type, DataType::Domain);
        assert!(matches!(eds.find_object(0x2000).unwrap().kind, ObjectKind::Var(_)));
    }

    #[test]
    fn compact_sub_obj_is_a_clear_error() {
        let err = Eds::parse("[6000]\nParameterName=X\nObjectType=0x8\nCompactSubObj=8\n").unwrap_err();
        assert!(err.to_string().contains("CompactSubObj"));
        assert!(err.to_string().contains("0x6000"));
    }

    #[test]
    fn unknown_data_type_is_an_error_with_context() {
        let err = Eds::parse("[2000]\nParameterName=X\nDataType=0x0010\nAccessType=ro\n").unwrap_err();
        assert!(err.to_string().contains("0x2000"));
        assert!(err.to_string().contains("0x0010"));
    }

    #[test]
    fn nodeid_expressions() {
        assert_eq!(parse_nodeid_expr("$NODEID+0x180"), Some((0x180, true)));
        assert_eq!(parse_nodeid_expr("0x200"), Some((0x200, false)));
        assert_eq!(parse_nodeid_expr("128"), Some((128, false)));
        assert_eq!(parse_nodeid_expr(""), None);
    }

    #[test]
    fn suspicious_defaults_are_diagnosed() {
        // Out of range for u8
        let eds = Eds::parse("[2000]\nParameterName=X\nDataType=0x0005\nAccessType=ro\nDefaultValue=300\n").unwrap();
        assert!(eds.diagnostics.iter().any(|d| d.contains("out of range")));

        // Unparsable integer
        let eds = Eds::parse("[2000]\nParameterName=X\nDataType=0x0006\nAccessType=ro\nDefaultValue=abc\n").unwrap();
        assert!(eds.diagnostics.iter().any(|d| d.contains("does not parse")));

        // Two's complement hex for signed types is accepted
        let eds = Eds::parse("[2000]\nParameterName=X\nDataType=0x0002\nAccessType=ro\nDefaultValue=0xFF\n").unwrap();
        assert!(eds.diagnostics.is_empty());

        // $NODEID expressions are exempt
        let eds = Eds::parse("[2000]\nParameterName=X\nDataType=0x0007\nAccessType=ro\nDefaultValue=$NODEID+0x80\n").unwrap();
        assert!(eds.diagnostics.is_empty());
    }

    #[test]
    fn array_count_mismatch_is_diagnosed_unless_dynamic() {
        const ARRAY: &str = "\
[3000]
ParameterName=A
ObjectType=0x8
SubNumber=0x3
[3000sub0]
ParameterName=count
DataType=0x0005
AccessType=ro
DefaultValue=DECLARED
[3000sub1]
ParameterName=e1
DataType=0x0006
AccessType=ro
[3000sub2]
ParameterName=e2
DataType=0x0006
AccessType=ro
";
        // Declared count disagrees with the highest element sub-index
        let eds = Eds::parse(&ARRAY.replace("DECLARED", "5")).unwrap();
        assert!(eds.diagnostics.iter().any(|d| d.contains("declares 5 elements")));

        // Matching count: clean
        let eds = Eds::parse(&ARRAY.replace("DECLARED", "2")).unwrap();
        assert!(eds.diagnostics.is_empty());

        // Zero means "dynamic count" (e.g. 0x1003) and is exempt
        let eds = Eds::parse(&ARRAY.replace("DECLARED", "0")).unwrap();
        assert!(eds.diagnostics.is_empty());
    }

    #[test]
    fn membership_mismatch_is_a_diagnostic_not_error() {
        let eds = Eds::parse(
            "[OptionalObjects]\nSupportedObjects=2\n1=0x2000\n2=0x3000\n\n[2000]\nParameterName=X\nDataType=0x0005\nAccessType=ro\n",
        )
        .unwrap();
        assert!(eds.diagnostics.iter().any(|d| d.contains("0x3000")));
    }
}
