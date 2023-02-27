use std::path::Path;
use std::io::Write;
use std::process::{
    Command,
    Stdio
};
use git2::Repository;

#[test]
fn validate_xml() {
   let output = Command::new(env!("CARGO_BIN_EXE_cargo-test-scripts"))
       .arg("--manifest-path")
       .arg(format!("{0}/tests/Cargo.toml", env!("CARGO_MANIFEST_DIR")))
       .stdout(Stdio::piped())
       .output()
       .expect("Failed to run");

    let tmpdir = env!("CARGO_TARGET_TMPDIR");

    let schema_repo_workspace = format!("{tmpdir}/JUnit-Schema");

    let path = Path::new(&schema_repo_workspace);

    if !path.exists() {
        Repository::clone("https://github.com/windyroad/JUnit-Schema.git", 
            &schema_repo_workspace).expect("Failed to clone");
    }
    
    let mut xmllint = Command::new("xmllint")
        .arg("--schema")
        .arg(format!("{schema_repo_workspace}/JUnit.xsd"))
        .arg("-")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to lint");

    let input = xmllint.stdin.as_mut().unwrap();
    input.write_all(&output.stdout).expect("Failed to write to stdin");
    drop(input);

    let output = xmllint.wait_with_output().expect("Failed to wait");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
}
