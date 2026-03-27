use std::collections::HashMap;

/// Parse a line and extract the commit hash and original line.
///
/// Input format: `<hash>\t<optional-title>`
///
/// Returns:
/// - `Ok(Some((hash, original_line)))` if the line is valid
/// - `Ok(None)` if the line is empty (should be skipped)
/// - `Err(line_number)` if the line has leading whitespace
pub fn parse_commit_line(line: &str, line_number: usize) -> Result<Option<(String, String)>, usize> {
    if line.is_empty() || line.trim().is_empty() {
        return Ok(None);
    }
    if line.starts_with(char::is_whitespace) {
        return Err(line_number);
    }
    let hash = line.split('\t').next().unwrap_or(line);
    Ok(Some((hash.to_string(), line.to_string())))
}

/// Parse multiple lines into a list of (hash, original_line) tuples.
///
/// Returns an error if any line has leading whitespace.
pub fn parse_commits(lines: &[String]) -> Result<Vec<(String, String)>, usize> {
    let mut result = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        match parse_commit_line(line, i + 1)? {
            Some(commit) => result.push(commit),
            None => {}
        }
    }
    Ok(result)
}

/// Sort commits by their position in the topological order.
///
/// Commits not found in the topo order are placed at the end.
pub fn sort_by_topo_order(
    commits: &mut [(String, String)],
    topo_order: &[&str],
) {
    let topo_map: HashMap<&str, usize> = topo_order
        .iter()
        .enumerate()
        .map(|(i, h)| (*h, i))
        .collect();

    commits.sort_by_key(|(hash, _)| topo_map.get(hash.as_str()).unwrap_or(&usize::MAX));
}
