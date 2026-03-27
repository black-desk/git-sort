use std::collections::HashMap;

/// Parse a line and extract the commit hash and original line.
///
/// Input format: `<hash>\t<optional-title>`
pub fn parse_commit_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let hash = line.split('\t').next().unwrap_or(line);
    Some((hash.to_string(), line.to_string()))
}

/// Parse multiple lines into a list of (hash, original_line) tuples.
pub fn parse_commits(lines: &[String]) -> Vec<(String, String)> {
    lines
        .iter()
        .filter_map(|line| parse_commit_line(line))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commit_line_with_title() {
        let line = "abc123\tInitial commit";
        let result = parse_commit_line(line);
        assert_eq!(
            result,
            Some(("abc123".to_string(), "abc123\tInitial commit".to_string()))
        );
    }

    #[test]
    fn test_parse_commit_line_without_title() {
        let line = "abc123";
        let result = parse_commit_line(line);
        assert_eq!(result, Some(("abc123".to_string(), "abc123".to_string())));
    }

    #[test]
    fn test_parse_commit_line_empty() {
        let line = "";
        let result = parse_commit_line(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_commit_line_whitespace_only() {
        let line = "   ";
        let result = parse_commit_line(line);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_commit_line_with_leading_whitespace() {
        let line = "  abc123\tTitle";
        let result = parse_commit_line(line);
        assert_eq!(
            result,
            Some(("abc123".to_string(), "abc123\tTitle".to_string()))
        );
    }

    #[test]
    fn test_parse_commits() {
        let lines = vec![
            "hash1\tCommit one".to_string(),
            "".to_string(),
            "hash2\tCommit two".to_string(),
        ];
        let result = parse_commits(&lines);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("hash1".to_string(), "hash1\tCommit one".to_string()));
        assert_eq!(result[1], ("hash2".to_string(), "hash2\tCommit two".to_string()));
    }

    #[test]
    fn test_sort_by_topo_order() {
        let mut commits = vec![
            ("hash3".to_string(), "hash3\tThird".to_string()),
            ("hash1".to_string(), "hash1\tFirst".to_string()),
            ("hash2".to_string(), "hash2\tSecond".to_string()),
        ];
        let topo_order = vec!["hash1", "hash2", "hash3"];
        sort_by_topo_order(&mut commits, &topo_order);
        assert_eq!(commits[0].0, "hash1");
        assert_eq!(commits[1].0, "hash2");
        assert_eq!(commits[2].0, "hash3");
    }

    #[test]
    fn test_sort_by_topo_order_with_unknown_commits() {
        let mut commits = vec![
            ("unknown".to_string(), "unknown\tUnknown".to_string()),
            ("hash1".to_string(), "hash1\tFirst".to_string()),
            ("hash2".to_string(), "hash2\tSecond".to_string()),
        ];
        let topo_order = vec!["hash2", "hash1"]; // Note: hash2 comes first in topo order
        sort_by_topo_order(&mut commits, &topo_order);
        assert_eq!(commits[0].0, "hash2");
        assert_eq!(commits[1].0, "hash1");
        assert_eq!(commits[2].0, "unknown"); // Unknown commits go to end
    }

    #[test]
    fn test_sort_preserves_original_line() {
        let mut commits = vec![
            ("hash2".to_string(), "hash2\tSecond commit".to_string()),
            ("hash1".to_string(), "hash1\tFirst commit".to_string()),
        ];
        let topo_order = vec!["hash1", "hash2"];
        sort_by_topo_order(&mut commits, &topo_order);
        assert_eq!(commits[0].1, "hash1\tFirst commit");
        assert_eq!(commits[1].1, "hash2\tSecond commit");
    }
}
