use std::io::Write;
use std::process::{Command, Stdio};

/// Test repository path (git submodule)
fn get_test_repo_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test-repo")
}

/// Commit hashes from test-repo:
/// A (Initial) -> B -> C (master)
///              \-> D -> E (feature)
mod commits {
    pub const A: &str = "9c46f2328d3804beabdd165dc0b6cab0185b00d6";
    pub const B: &str = "178293e6c71fc12ed55e7ae8e4f24e086ee524e9";
    pub const C: &str = "2e5d0971eaf78226a2c1b416f9f63a66bbab17ad";
    pub const D: &str = "e17208ca639046c4e254a513ba64b19b013f426c";
    pub const E: &str = "97dadd722f8764abd01c6a9103f51787c0d753fb";
}

fn get_binary_path() -> std::path::PathBuf {
    std::env::var("CARGO_BIN_EXE_git-sort")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_exe()
                .expect("Failed to get current exe")
                .parent()
                .expect("Failed to get parent")
                .join("git-sort")
        })
}

fn run_git_sort(args: &[&str], input: &str) -> (bool, String) {
    let binary = get_binary_path();
    let repo = get_test_repo_path();

    let mut child = Command::new(&binary)
        .args(args)
        .current_dir(&repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn git-sort");

    {
        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write");
    }

    let result = child.wait_with_output().expect("Failed to get output");
    let stdout = String::from_utf8_lossy(&result.stdout).to_string();

    (result.status.success(), stdout)
}

#[test]
fn test_basic_sort() {
    // Create input with commits in wrong order
    let input = format!("{}\tC\n{}\tA\n{}\tB\n", commits::C, commits::A, commits::B);

    let (success, stdout) = run_git_sort(&[], &input);
    assert!(success);

    // Should be sorted in topo order (newest first): C, B, A
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with(commits::C));
    assert!(lines[1].starts_with(commits::B));
    assert!(lines[2].starts_with(commits::A));
}

#[test]
fn test_empty_input() {
    let (success, stdout) = run_git_sort(&[], "\n\n");
    assert!(success);
    assert!(stdout.trim().is_empty());
}

#[test]
fn test_single_commit() {
    let input = format!("{}\tA\n", commits::A);
    let (success, stdout) = run_git_sort(&[], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with(commits::A));
}

#[test]
fn test_commit_not_on_reference_branch() {
    // D is on feature branch, not on master
    let input = format!("{}\tD\n{}\tA\n{}\tB\n", commits::D, commits::A, commits::B);

    let binary = get_binary_path();
    let repo = get_test_repo_path();

    let mut child = Command::new(&binary)
        .current_dir(&repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn git-sort");

    {
        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write");
    }

    let result = child.wait_with_output().expect("Failed to get output");

    // Should fail with error
    assert!(!result.status.success());

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("not reachable"));
    assert!(stderr.contains(commits::D));
}

#[test]
fn test_with_reference_option() {
    // On feature branch: E is newest, then D, then B
    let input = format!("{}\tB\n{}\tE\n{}\tD\n", commits::B, commits::E, commits::D);

    let (success, stdout) = run_git_sort(&["--reference", "feature"], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with(commits::E));
    assert!(lines[1].starts_with(commits::D));
    assert!(lines[2].starts_with(commits::B));
}

#[test]
fn test_output_to_file() {
    let input = format!("{}\tA\n{}\tB\n", commits::A, commits::B);
    let output_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    let output_path = output_file.path().to_str().unwrap();

    let binary = get_binary_path();
    let repo = get_test_repo_path();

    let mut child = Command::new(&binary)
        .args(["-o", output_path])
        .current_dir(&repo)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn git-sort");

    {
        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write");
    }

    let result = child.wait().expect("Failed to wait");
    assert!(result.success());

    let output_content = std::fs::read_to_string(output_path).expect("Failed to read output");
    let lines: Vec<&str> = output_content.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(commits::B)); // B is newer
    assert!(lines[1].starts_with(commits::A));
}

#[test]
fn test_input_without_title() {
    // Input without title (just hashes)
    let input = format!("{}\n{}\n", commits::B, commits::A);

    let (success, stdout) = run_git_sort(&[], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(commits::B));
    assert!(lines[1].starts_with(commits::A));
}

#[test]
fn test_input_with_whitespace_lines() {
    // Input with blank lines (should be skipped)
    let input = format!("{}\tB\n\n{}\tA\n   \n", commits::B, commits::A);

    let (success, stdout) = run_git_sort(&[], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(commits::B));
    assert!(lines[1].starts_with(commits::A));
}

#[test]
fn test_input_with_leading_whitespace_error() {
    // Input with leading whitespace (should error)
    let input = format!("{}\tA\n  {}\n", commits::A, commits::A);

    let binary = get_binary_path();
    let repo = get_test_repo_path();

    let mut child = Command::new(&binary)
        .current_dir(&repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn git-sort");

    {
        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write");
    }

    let result = child.wait_with_output().expect("Failed to get output");
    assert!(!result.status.success());

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains("leading whitespace"));
    assert!(stderr.contains("line 2"));
}

#[test]
fn test_output_preserves_title() {
    // Input with titles
    let input = format!(
        "{}\tFirst commit\n{}\tSecond commit\n",
        commits::A, commits::B
    );

    let (success, stdout) = run_git_sort(&[], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    // Verify titles are preserved
    assert!(lines[0].contains("Second commit"));
    assert!(lines[1].contains("First commit"));
}
