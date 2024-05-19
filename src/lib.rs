// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::str;

/// Returns the URL of the repository.
///
/// # Errors
///
/// This function will return an error if the repository remote cannot be
/// found or URL is not valid UTF-8.
pub fn gix_repo_url(
    repo: &gix::Repository,
    direction: gix::remote::Direction,
) -> Result<Option<(String, String)>, Box<dyn Error>> {
    if let Some(remote) = repo.find_default_remote(direction).transpose()? {
        if let Some(url) = remote.url(direction) {
            if let Some(host) = url.host_argument_safe() {
                if let Some(path) = url.path_argument_safe() {
                    let path = path.strip_suffix(b".git").unwrap_or(path);
                    return Ok(Some((host.to_string(), str::from_utf8(path)?.to_string())));
                }
            }
        }
    }
    Ok(None)
}

/// Search for `text` in the file at `path` and return the line numbers.
///
/// # Errors
///
/// This function will return an error if the file cannot be read or if no
/// lines contain `text`.
pub fn search_lines(path: &Path, text: &str) -> io::Result<Vec<usize>> {
    let reader = BufReader::new(File::open(path)?);
    let mut text_lines = text.lines().peekable();
    let line_nums: Vec<_> = reader
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            if let Some(text_line) = text_lines.peek() {
                if line.ok()?.contains(text_line) {
                    text_lines.next();
                    return Some(i + 1);
                }
            }
            None
        })
        .collect();
    if line_nums.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("file {} does not contain string {text}", path.display()),
        ));
    }
    Ok(line_nums)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_gix_repo_url() {
        let path = gix_testtools::scripted_fixture_read_only("create-repo").unwrap();
        let repo = gix::open(path).unwrap();
        assert_eq!(
            gix_repo_url(&repo, gix::remote::Direction::Fetch).unwrap(),
            Some((
                "github.test.com".to_string(),
                "matthewdargan/repo".to_string()
            ))
        );
    }

    #[test]
    fn test_gix_repo_url_no_git() {
        let path = gix_testtools::scripted_fixture_read_only("create-repo-no-git").unwrap();
        let repo = gix::open(path).unwrap();
        assert_eq!(
            gix_repo_url(&repo, gix::remote::Direction::Fetch).unwrap(),
            Some(("github.com".to_string(), "repo".to_string()))
        );
    }

    #[test]
    fn test_gix_repo_url_no_remote() {
        let path = tempdir().unwrap().into_path().join("repo");
        let repo = gix::init(path).unwrap();
        assert_eq!(
            gix_repo_url(&repo, gix::remote::Direction::Fetch).unwrap(),
            None
        );
    }

    #[test]
    fn test_gix_repo_url_invalid_host() {
        let path = gix_testtools::scripted_fixture_read_only("create-repo-invalid-host").unwrap();
        let repo = gix::open(path).unwrap();
        assert!(gix_repo_url(&repo, gix::remote::Direction::Fetch).is_err());
    }

    #[test]
    fn test_gix_repo_url_invalid_path() {
        let path = gix_testtools::scripted_fixture_read_only("create-repo-invalid-path").unwrap();
        let repo = gix::open(path).unwrap();
        assert!(gix_repo_url(&repo, gix::remote::Direction::Fetch).is_err());
    }

    #[test]
    fn test_search_lines() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"foo\nbar\nbaz\nbar").unwrap();
        assert_eq!(search_lines(file.path(), "bar").unwrap(), vec![2]);
    }

    #[test]
    fn test_search_lines_continuous() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"foo1\nfoo2\nfoo3\nfoo4\nfoo5\nfoo6")
            .unwrap();
        assert_eq!(
            search_lines(file.path(), "foo2\nfoo3\nfoo4\nfoo5").unwrap(),
            vec![2, 3, 4, 5]
        );
    }

    #[test]
    fn test_search_lines_no_file() {
        let file = NamedTempFile::new().unwrap();
        assert!(search_lines(file.path(), "").is_err());
    }

    #[test]
    fn test_search_lines_not_found() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"foo\nbar\nbaz\nbar").unwrap();
        assert!(search_lines(file.path(), "not found").is_err());
    }
}
