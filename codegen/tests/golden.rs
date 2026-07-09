//! Golden tests: generation output for the checked-in fixtures must match
//! the checked-in golden files (regenerate deliberately via
//! `cargo run --example generate_golden`), and the golden output must
//! compile and behave against the real canopen-async crate.

use canopen_async::dict::{Domain, SdoArray, SdoEntry, VisibleString};
use canopen_async::pdo::PdoPayload as _;

#[allow(dead_code)]
mod enyring {
    include!("golden/bat_enyring_dict.rs");
}

#[allow(dead_code)]
mod pdo_device {
    include!("golden/pdo_device_dict.rs");
}

fn generate(fixture: &str) -> String {
    let path = format!("{}/tests/fixtures/{fixture}", env!("CARGO_MANIFEST_DIR"));
    let src = canopen_async_codegen::read_eds_file(path).unwrap();
    canopen_async_codegen::Config::new(fixture, &src).generate().unwrap()
}

#[test]
fn enyring_output_matches_golden() {
    assert_eq!(
        generate("bat_enyring.eds"),
        include_str!("golden/bat_enyring_dict.rs"),
        "generator output drifted; review and regenerate via `cargo run --example generate_golden`"
    );
}

#[test]
fn pdo_device_output_matches_golden() {
    assert_eq!(generate("pdo_device.eds"), include_str!("golden/pdo_device_dict.rs"));
}

#[test]
fn enyring_entries_have_correct_addresses_and_types() {
    // The entries the enyring driver uses today, spot-checked against
    // its hand-written sdo_dict.rs addresses.
    assert_eq!(enyring::state::STATE_OF_CHARGE.index, 0x4241);
    assert_eq!(enyring::state::STATE_OF_CHARGE.sub, 0x01);
    let _: SdoEntry<u8> = enyring::state::STATE_OF_CHARGE;
    let _: SdoEntry<i32> = enyring::state::CURRENT;
    let _: SdoEntry<u16> = enyring::state::PACK_VOLTAGE;
    let _: SdoEntry<u64> = enyring::BATTERY_ERROR;
    let _: SdoEntry<i64> = enyring::BATTERY_VALIDITY;
    let _: SdoEntry<u32> = enyring::STATUS_WORD;
    let _: SdoEntry<VisibleString> = enyring::BATTERY_UID;
    let _: SdoEntry<Domain> = enyring::ACCOUNT_UID;
    let _: SdoEntry<Domain> = enyring::station_auth::NONCE;
    let _: SdoEntry<Domain> = enyring::FW_UPDATE;

    let cells: SdoArray<u16> = enyring::cells_voltage::ENTRIES;
    assert_eq!(cells.index, 0x4242);
    assert_eq!(cells.len, 13);
    assert_eq!(cells.entry(3).sub, 3);

    let bikes: SdoArray<Domain> = enyring::accepted_bikes::ENTRIES;
    assert_eq!(bikes.len, 100);
}

#[test]
fn generated_tpdo_decodes_and_rpdo_encodes() {
    // Tpdo1: Voltage u16 @0, one dummy padding byte @2, Current i32 @3
    // (7 bytes total). The padding byte (0xEE) must not leak into a field.
    let data = [0x34, 0x12, 0xEE, 0x3C, 0xFB, 0xFF, 0xFF, 0];
    let tpdo = pdo_device::Tpdo1::decode(&data).unwrap();
    assert_eq!(tpdo.voltage, 0x1234);
    assert_eq!(tpdo.current, -1220);

    // Too-short payload is rejected
    assert!(pdo_device::Tpdo1::decode(&data[..6]).is_none());

    // Tpdo2: Flags u8
    let tpdo2 = pdo_device::Tpdo2::decode(&[0xA5]).unwrap();
    assert_eq!(tpdo2.flags, 0xA5);

    // Rpdo1: Target current i16
    let rpdo = pdo_device::Rpdo1 { target_current: -300 };
    let (bytes, len) = rpdo.to_bytes();
    assert_eq!(len, 2);
    assert_eq!(bytes[..2], (-300i16).to_le_bytes());
}

#[test]
fn limits_appear_in_generated_docs() {
    let code = generate("pdo_device.eds");
    assert!(code.contains("range `-5000`..=`5000`"));
}

#[test]
fn build_dictionary_writes_out_dir_file_for_the_macro() {
    let out_dir = std::env::temp_dir().join(format!("canopen-codegen-test-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).unwrap();

    let eds = format!("{}/tests/fixtures/bat_enyring.eds", env!("CARGO_MANIFEST_DIR"));
    let (file, diagnostics) = canopen_async_codegen::generate_dictionary_file(
        "ENYRING",
        std::path::Path::new(&eds),
        &out_dir,
        |config| config,
    )
    .unwrap();

    assert_eq!(file.file_name().unwrap(), "canopen_dictionary_ENYRING.rs");
    assert!(std::fs::read_to_string(&file).unwrap().contains("SdoEntry"));
    assert!(diagnostics.is_empty());

    // Invalid labels are rejected (they become env-var/file names)
    assert!(
        canopen_async_codegen::generate_dictionary_file(
            "2 bad name",
            std::path::Path::new(&eds),
            &out_dir,
            |config| config,
        )
        .is_err()
    );

    std::fs::remove_dir_all(&out_dir).ok();
}
