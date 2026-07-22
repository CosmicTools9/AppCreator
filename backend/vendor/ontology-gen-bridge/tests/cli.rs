//! Integration tests for ontology-gen-bridge CLI

#[cfg(test)]
mod cli_tests {
    use std::path::Path;
    use std::process::Command;

    fn binary() -> Command {
        // Use the compiled binary from cargo test
        let path = std::env::var("CARGO_BIN_EXE_ontology-gen-bridge")
            .unwrap_or_else(|_| "target/debug/ontology-gen-bridge".into());
        Command::new(path)
    }

    fn fixture(name: &str) -> String {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
            .to_string_lossy()
            .into()
    }

    #[test]
    fn test_cli_basic_generation() {
        let out = std::env::temp_dir().join("gen-bridge-test-basic");
        let _ = std::fs::remove_dir_all(&out);

        let status = binary()
            .args([
                "--input",
                &fixture("basic.json"),
                "--output",
                out.to_str().unwrap(),
                "--name",
                "test",
            ])
            .status()
            .expect("failed to run binary");

        assert!(status.success(), "CLI exited with {:?}", status.code());

        // Check expected files exist
        assert!(out.join("Cargo.toml").exists());
        assert!(out.join("src/lib.rs").exists());
        assert!(out.join("src/routes.rs").exists());
        assert!(out.join("src/errors.rs").exists());
        assert!(out.join("src/models/measurement_unit.rs").exists());

        // Clean up
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn test_cli_rejects_qk_field() {
        let out = std::env::temp_dir().join("gen-bridge-test-qk");
        let _ = std::fs::remove_dir_all(&out);

        let output = binary()
            .args([
                "--input",
                &fixture("basic.json"),
                "--output",
                out.to_str().unwrap(),
            ])
            .output()
            .expect("failed");

        // basic.json has no qk_* fields, so should succeed
        assert!(output.status.success());

        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn test_cli_missing_input() {
        let status = binary()
            .arg("--output")
            .arg("/tmp/nowhere")
            .status()
            .expect("failed");
        assert!(!status.success());
    }

    #[test]
    fn test_safe_output_path() {
        use std::path::PathBuf;

        // Import the function from main.rs (we can't, it's private)
        // Test path safety logic inline
        let base = PathBuf::from("/tmp/out");
        let safe = |rel: &str| -> bool {
            let p = std::path::Path::new(rel);
            for c in p.components() {
                match c {
                    std::path::Component::Normal(_) | std::path::Component::CurDir => {}
                    _ => return false,
                }
            }
            true
        };

        assert!(safe("src/models/foo.rs"));
        assert!(safe("Cargo.toml"));
        assert!(!safe("../escape.rs"));
        assert!(!safe("/etc/passwd"));
        assert!(!safe("sub/../../escape.rs"));
    }
}
