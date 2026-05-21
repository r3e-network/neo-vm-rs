use std::{fs, path::Path};

const MAX_INTERPRETER_SOURCE_LINES: usize = 1_000;
const MAX_EXECUTOR_SOURCE_LINES: usize = 2_600;

#[test]
fn interpreter_sources_stay_small_enough_to_review() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interpreter");
    let mut oversized = Vec::new();

    collect_oversized_sources(&root, &mut oversized);

    assert!(
        oversized.is_empty(),
        "interpreter source files exceed their review-size limits: {oversized:?}"
    );
}

fn collect_oversized_sources(dir: &Path, oversized: &mut Vec<(String, usize)>) {
    for entry in fs::read_dir(dir).expect("interpreter directory should be readable") {
        let entry = entry.expect("interpreter entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_oversized_sources(&path, oversized);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let source = fs::read_to_string(&path).expect("source file should be UTF-8");
        let line_count = source.lines().count();
        let limit = if path.file_name().and_then(|name| name.to_str()) == Some("executor.rs") {
            MAX_EXECUTOR_SOURCE_LINES
        } else {
            MAX_INTERPRETER_SOURCE_LINES
        };

        if line_count > limit {
            oversized.push((
                path.strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")))
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
                line_count,
            ));
        }
    }
}
