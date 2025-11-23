// tests/01_ignite_project.rs
//
// RED PHASE: This test MUST fail initially
//
// This is the first test for AllFrame, as specified in PRD_01.md.
// It verifies that the `allframe ignite` command creates a compilable project
// with all features enabled.
//
// Expected behavior:
// 1. Run `allframe ignite testproject`
// 2. Command succeeds
// 3. Generated project can be compiled
// 4. Generated project tests pass

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn ignite_creates_compilable_project_with_all_features() {
    // Create a temporary directory for the test project
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().join("testproject");

    // Run `allframe ignite testproject` command
    let mut cmd = Command::cargo_bin("allframe").expect("Failed to find allframe binary");

    cmd.arg("ignite")
        .arg(&project_path)
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AllFrame project created"))
        .stdout(predicate::str::contains("testproject"));

    // Verify the project was created
    assert!(
        project_path.exists(),
        "Project directory should exist at {:?}",
        project_path
    );

    // Verify Cargo.toml exists
    let cargo_toml = project_path.join("Cargo.toml");
    assert!(
        cargo_toml.exists(),
        "Cargo.toml should exist in generated project"
    );

    // Verify Cargo.toml contains allframe dependency
    let cargo_content = fs::read_to_string(&cargo_toml)
        .expect("Failed to read Cargo.toml");
    assert!(
        cargo_content.contains("allframe"),
        "Cargo.toml should contain allframe dependency"
    );

    // Verify src/main.rs exists
    let main_rs = project_path.join("src/main.rs");
    assert!(
        main_rs.exists(),
        "src/main.rs should exist in generated project"
    );

    // Try to compile the generated project
    let compile_output = std::process::Command::new("cargo")
        .arg("build")
        .current_dir(&project_path)
        .output()
        .expect("Failed to run cargo build");

    assert!(
        compile_output.status.success(),
        "Generated project should compile successfully. stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    // Run tests in the generated project
    let test_output = std::process::Command::new("cargo")
        .arg("test")
        .current_dir(&project_path)
        .output()
        .expect("Failed to run cargo test");

    assert!(
        test_output.status.success(),
        "Generated project tests should pass. stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&test_output.stdout),
        String::from_utf8_lossy(&test_output.stderr)
    );

    // Clean up is automatic when temp_dir goes out of scope
}

#[test]
fn ignite_fails_with_invalid_project_name() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Try to create a project with an invalid name (contains spaces)
    let mut cmd = Command::cargo_bin("allframe").expect("Failed to find allframe binary");

    cmd.arg("ignite")
        .arg("invalid project name")
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid project name"));
}

#[test]
fn ignite_fails_if_directory_already_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().join("existing");

    // Create the directory first
    fs::create_dir(&project_path).expect("Failed to create directory");

    // Try to create a project in the existing directory
    let mut cmd = Command::cargo_bin("allframe").expect("Failed to find allframe binary");

    cmd.arg("ignite")
        .arg(&project_path)
        .current_dir(temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn ignite_creates_project_with_correct_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().join("structured");

    // Run `allframe ignite structured`
    Command::cargo_bin("allframe")
        .expect("Failed to find allframe binary")
        .arg("ignite")
        .arg(&project_path)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify expected directory structure
    let expected_files = vec![
        "Cargo.toml",
        "src/main.rs",
        "src/domain/mod.rs",
        "src/application/mod.rs",
        "src/infrastructure/mod.rs",
        "src/presentation/mod.rs",
        ".gitignore",
        "README.md",
    ];

    for file in expected_files {
        let file_path = project_path.join(file);
        assert!(
            file_path.exists(),
            "Expected file should exist: {}",
            file_path.display()
        );
    }
}

#[test]
fn ignite_creates_project_with_all_feature_flags() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = temp_dir.path().join("featured");

    // Run `allframe ignite featured --all-features`
    Command::cargo_bin("allframe")
        .expect("Failed to find allframe binary")
        .arg("ignite")
        .arg(&project_path)
        .arg("--all-features")
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify Cargo.toml has all features
    let cargo_toml = project_path.join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_toml)
        .expect("Failed to read Cargo.toml");

    let expected_features = vec!["di", "openapi", "otel", "router", "cqrs", "mcp"];

    for feature in expected_features {
        assert!(
            cargo_content.contains(feature),
            "Cargo.toml should contain feature: {}",
            feature
        );
    }
}
