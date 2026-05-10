//! Optional fixture corpus integration test.
//!
//! Place `.gdtf` files in `tests/fixture_corpus/` (workspace root) and run
//! `cargo test -p stagelx-gdtf --test corpus`.
//!
//! If no files are present, the test prints a message and passes.

use std::path::Path;

#[test]
fn corpus_parses_without_panic() {
    // The corpus directory lives at the workspace root, not inside the crate.
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixture_corpus");

    if !corpus_dir.exists() {
        println!("Skipping corpus test — directory does not exist: {}", corpus_dir.display());
        return;
    }

    let entries: Vec<_> = std::fs::read_dir(&corpus_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("gdtf"))
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        println!("Skipping corpus test — no .gdtf files in {}", corpus_dir.display());
        return;
    }

    let mut passed = 0usize;
    let mut failed = 0usize;

    for entry in entries {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  FAIL  {} — cannot read file: {e}", file_name);
                failed += 1;
                continue;
            }
        };

        match stagelx_gdtf::parse_gdtf(&data) {
            Ok(ft) => {
                if ft.dmx_modes.is_empty() {
                    eprintln!("  FAIL  {} — parsed OK but has zero DMX modes", file_name);
                    failed += 1;
                } else {
                    println!(
                        "  OK    {} — {} mode(s), {} channel(s) total",
                        file_name,
                        ft.dmx_modes.len(),
                        ft.dmx_modes.iter().map(|m| m.channels.len()).sum::<usize>()
                    );
                    passed += 1;
                }
            }
            Err(e) => {
                eprintln!("  FAIL  {} — parse error: {e}", file_name);
                failed += 1;
            }
        }
    }

    println!("\nCorpus summary: {passed} passed, {failed} failed out of {} files", passed + failed);
    assert_eq!(failed, 0, "{} corpus fixture(s) failed to parse", failed);
}
