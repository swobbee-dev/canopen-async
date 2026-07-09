//! Regenerate the golden test outputs after intentional emitter changes:
//! `cargo run --example generate_golden` from `codegen/`.

fn main() {
    for (fixture, golden) in [
        ("bat_enyring.eds", "bat_enyring_dict.rs"),
        ("pdo_device.eds", "pdo_device_dict.rs"),
    ] {
        let src = canopen_async_codegen::read_eds_file(format!("tests/fixtures/{fixture}")).unwrap();
        let (code, diags) = canopen_async_codegen::Config::new(fixture, &src)
            .generate_with_diagnostics()
            .unwrap();
        std::fs::write(format!("tests/golden/{golden}"), &code).unwrap();
        eprintln!("{fixture}: {} diagnostics", diags.len());
        for d in diags {
            eprintln!("  {d}");
        }
    }
}
