use std::io::Write;
use std::process::{Command, Stdio};

/// Helper to create a test git repository with commits
fn create_test_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let path = dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .expect("Failed to init git repo");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .expect("Failed to config email");

    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(path)
        .output()
        .expect("Failed to config name");

    dir
}

/// Create a commit and return its hash
fn create_commit(repo: &std::path::Path, msg: &str) -> String {
    Command::new("git")
        .args(["commit", "--allow-empty", "-m", msg])
        .current_dir(repo)
        .output()
        .expect("Failed to create commit");

    let hash_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .expect("Failed to get commit hash");

    String::from_utf8_lossy(&hash_output.stdout).trim().to_string()
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

fn run_git_sort(repo: &std::path::Path, args: &[&str], input: &str) -> (bool, String) {
    let binary = get_binary_path();

    let mut child = Command::new(&binary)
        .args(args)
        .current_dir(repo)
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
    let repo = create_test_repo();
    let path = repo.path();

    // Create commits: A -> B -> C
    let a = create_commit(path, "A");
    let b = create_commit(path, "B");
    let c = create_commit(path, "C");

    // Create input with commits in wrong order
    let input = format!("{}\tC\n{}\tA\n{}\tB\n", c, a, b);

    let (success, stdout) = run_git_sort(path, &[], &input);
    assert!(success);

    // Should be sorted in topo order (newest first): C, B, A
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with(&c));
    assert!(lines[1].starts_with(&b));
    assert!(lines[2].starts_with(&a));
}

#[test]
fn test_empty_input() {
    let repo = create_test_repo();
    let path = repo.path();

    let (success, stdout) = run_git_sort(path, &[], "\n\n");
    assert!(success);
    assert!(stdout.trim().is_empty());
}

#[test]
fn test_single_commit() {
    let repo = create_test_repo();
    let path = repo.path();

    let a = create_commit(path, "A");

    let input = format!("{}\tA\n", a);
    let (success, stdout) = run_git_sort(path, &[], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with(&a));
}

#[test]
fn test_commit_not_on_reference_branch() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create: A -> B (master)
    //              \-> C (feature)
    let a = create_commit(path, "A");
    let b = create_commit(path, "B");

    Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(path)
        .output()
        .expect("Failed to create branch");

    let c = create_commit(path, "C");

    Command::new("git")
        .args(["checkout", "master"])
        .current_dir(path)
        .output()
        .expect("Failed to checkout");

    // Input includes C which is not on master
    let input = format!("{}\tC\n{}\tA\n{}\tB\n", c, a, b);

    let binary = get_binary_path();
    let mut child = Command::new(&binary)
        .current_dir(path)
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
    assert!(stderr.contains(&c));
}

#[test]
fn test_with_reference_option() {
    let repo = create_test_repo();
    let path = repo.path();

    // Create: A -> B (master)
    //              \-> C -> D (feature)
    let _a = create_commit(path, "A");
    let b = create_commit(path, "B");

    Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(path)
        .output()
        .expect("Failed to create branch");

    let c = create_commit(path, "C");
    let d = create_commit(path, "D");

    // Input with --reference feature should sort by feature branch
    let input = format!("{}\tB\n{}\tD\n{}\tC\n", b, d, c);

    let (success, stdout) = run_git_sort(path, &["--reference", "feature"], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    // On feature branch: D is newest, then C, then B
    assert!(lines[0].starts_with(&d));
    assert!(lines[1].starts_with(&c));
    assert!(lines[2].starts_with(&b));
}

#[test]
fn test_output_to_file() {
    let repo = create_test_repo();
    let path = repo.path();

    let a = create_commit(path, "A");
    let b = create_commit(path, "B");

    let input = format!("{}\tA\n{}\tB\n", a, b);
    let output_file = path.join("output.txt");

    let binary = get_binary_path();
    let mut child = Command::new(&binary)
        .args(["-o", output_file.to_str().unwrap()])
        .current_dir(path)
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn git-sort");

    {
        let mut stdin = child.stdin.take().expect("Failed to get stdin");
        stdin.write_all(input.as_bytes()).expect("Failed to write");
    }

    let result = child.wait().expect("Failed to wait");
    assert!(result.success());

    let output_content = std::fs::read_to_string(&output_file).expect("Failed to read output");
    let lines: Vec<&str> = output_content.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(&b)); // B is newer
    assert!(lines[1].starts_with(&a));
}

#[test]
fn test_input_without_title() {
    let repo = create_test_repo();
    let path = repo.path();

    let a = create_commit(path, "A");
    let b = create_commit(path, "B");

    // Input without title (just hashes)
    let input = format!("{}\n{}\n", b, a);

    let (success, stdout) = run_git_sort(path, &[], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(&b));
    assert!(lines[1].starts_with(&a));
}

#[test]
fn test_input_with_whitespace_lines() {
    let repo = create_test_repo();
    let path = repo.path();

    let a = create_commit(path, "A");
    let b = create_commit(path, "B");

    // Input with blank lines (should be skipped)
    let input = format!("{}\tB\n\n{}\tA\n   \n", b, a);

    let (success, stdout) = run_git_sort(path, &[], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with(&b));
    assert!(lines[1].starts_with(&a));
}

#[test]
fn test_input_with_leading_whitespace_error() {
    let repo = create_test_repo();
    let path = repo.path();

    let a = create_commit(path, "A");

    // Input with leading whitespace (should error)
    let input = format!("{}\tA\n  {}\n", a, a);

    let binary = get_binary_path();
    let mut child = Command::new(&binary)
        .current_dir(path)
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
    let repo = create_test_repo();
    let path = repo.path();

    let a = create_commit(path, "A");
    let b = create_commit(path, "B");

    // Input with titles
    let input = format!("{}\tFirst commit\n{}\tSecond commit\n", a, b);

    let (success, stdout) = run_git_sort(path, &[], &input);
    assert!(success);

    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    // Verify titles are preserved
    assert!(lines[0].contains("Second commit"));
    assert!(lines[1].contains("First commit"));
}
