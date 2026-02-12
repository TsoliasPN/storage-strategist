use anyhow::Result;
use std::path::PathBuf;
use storage_strategist_core::scan::ScanOptions;
use storage_strategist_core::scan::compare_backends;

#[test]
fn test_backend_parity_on_fixtures() -> Result<()> {
    // Resolve the fixtures directory relative to the crate root.
    // cargo test runs with the current directory set to the package root (crates/core).
    // So fixtures are at ../../fixtures
    let mut fixtures_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fixtures_path.pop(); // crates
    fixtures_path.pop(); // project root
    fixtures_path.push("fixtures");

    if !fixtures_path.exists() {
        // Fallback: if running from workspace root
        fixtures_path = PathBuf::from("fixtures");
    }
    
    // Ensure we found the fixtures
    assert!(fixtures_path.exists(), "Fixtures directory not found at {:?}", fixtures_path);

    let options = ScanOptions {
        paths: vec![fixtures_path],
        max_depth: None,
        // Ensure we test with a clean slate
        incremental_cache: false,
        ..Default::default()
    };

    let parity = compare_backends(&options)?;

    println!("Parity Result: {:?}", parity);

    assert!(
        parity.within_tolerance,
        "Backends diverged beyond tolerance! Delta: files={}, bytes={}",
        parity.scanned_files_delta,
        parity.scanned_bytes_delta
    );

    // Stricter check for fixtures: we expect EXACT parity (0 delta) because standard files
    // shouldn't have ambiguity between walkers.
    assert_eq!(parity.scanned_files_delta, 0, "File count mismatch");
    assert_eq!(parity.scanned_bytes_delta, 0, "Byte count mismatch");

    Ok(())
}
