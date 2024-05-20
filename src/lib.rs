// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug)]
pub struct UrlGenerationArgs {
    pub link_opts: LinkOptions,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum LinkOptions {
    Lines(usize, Option<usize>),
    Search(String),
    Empty,
}

/// Returns the GitHub blob URL for the file at `path` with the given
/// `link_opts`.
///
/// # Errors
///
/// This function will return an error if the repository cannot be found or
/// if the file cannot be read.
pub fn blob_url(cli: &UrlGenerationArgs) -> Result<String, Box<dyn Error>> {
    let mut url = {
        let mut abs_path = fs::canonicalize(&cli.path)?;
        if abs_path.is_file() {
            abs_path.pop();
        }
        let repo = gix::discover(&abs_path)?;
        let (host, path) =
            gix_repo_url(&repo, gix::remote::Direction::Fetch)?.ok_or("failed to get repo URL")?;
        let commit = repo.rev_parse_single("HEAD")?;
        let rel_path = repo
            .prefix()?
            .ok_or("failed to get repo relative path")?
            .join(&cli.path);
        format!("https://{host}/{path}/blob/{commit}/{}", rel_path.display())
    };
    match &cli.link_opts {
        LinkOptions::Lines(l1, l2) => {
            url.push_str(&format!("#L{l1}"));
            if let Some(l2) = l2 {
                url.push_str(&format!("-L{l2}"));
            }
        }
        LinkOptions::Search(search) => {
            let search = if search == "-" {
                io::read_to_string(io::stdin())?
            } else {
                search.clone()
            };
            let line_nums = search_lines(cli.path.as_path(), &search)?;
            url.push_str(&format!(
                "#L{}",
                line_nums.first().ok_or("no line numbers found")?
            ));
            if line_nums.len() > 1 {
                url.push_str(&format!(
                    "-L{}",
                    line_nums.last().ok_or("no line numbers found")?
                ));
            }
        }
        LinkOptions::Empty => {}
    }
    Ok(url)
}

/// Returns the repository URL.
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
    use serial_test::serial;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    #[serial]
    fn test_blob_url_empty() {
        let path = gix_testtools::scripted_fixture_read_only("create-readme-repo").unwrap();
        let _keep = gix_testtools::set_current_dir(path);
        let args = UrlGenerationArgs {
            link_opts: LinkOptions::Empty,
            path: "README.md".into(),
        };
        let re = regex::Regex::new(
            r"https://github\.test\.com/matthewdargan/repo/blob/[0-9a-f]{40}/README\.md",
        )
        .unwrap();
        assert!(re.is_match(blob_url(&args).unwrap().as_str()));
    }

    #[test]
    #[serial]
    fn test_blob_url_l1() {
        let path = gix_testtools::scripted_fixture_read_only("create-readme-repo").unwrap();
        let _keep = gix_testtools::set_current_dir(path);
        let args = UrlGenerationArgs {
            link_opts: LinkOptions::Lines(1, None),
            path: "README.md".into(),
        };
        let re = regex::Regex::new(
            r"https://github\.test\.com/matthewdargan/repo/blob/[0-9a-f]{40}/README\.md#L1",
        )
        .unwrap();
        assert!(re.is_match(blob_url(&args).unwrap().as_str()));
    }

    #[test]
    #[serial]
    fn test_blob_url_l1_l2() {
        let path = gix_testtools::scripted_fixture_read_only("create-readme-repo").unwrap();
        let _keep = gix_testtools::set_current_dir(path);
        let args = UrlGenerationArgs {
            link_opts: LinkOptions::Lines(3, Some(8)),
            path: "README.md".into(),
        };
        let re = regex::Regex::new(
            r"https://github\.test\.com/matthewdargan/repo/blob/[0-9a-f]{40}/README\.md#L3-L8",
        )
        .unwrap();
        assert!(re.is_match(blob_url(&args).unwrap().as_str()));
    }

    #[test]
    #[serial]
    fn test_blob_url_s() {
        let path = gix_testtools::scripted_fixture_read_only("create-readme-repo").unwrap();
        let _keep = gix_testtools::set_current_dir(path);
        let args = UrlGenerationArgs {
            link_opts: LinkOptions::Search("foo1".into()),
            path: "README.md".into(),
        };
        let re = regex::Regex::new(
            r"https://github\.test\.com/matthewdargan/repo/blob/[0-9a-f]{40}/README\.md#L1",
        )
        .unwrap();
        assert!(re.is_match(blob_url(&args).unwrap().as_str()));
    }

    #[test]
    #[serial]
    fn test_blob_url_s_continuous() {
        let path = gix_testtools::scripted_fixture_read_only("create-readme-repo").unwrap();
        let _keep = gix_testtools::set_current_dir(path);
        let args = UrlGenerationArgs {
            link_opts: LinkOptions::Search("foo1\nfoo2\nfoo3".into()),
            path: "README.md".into(),
        };
        let re = regex::Regex::new(
            r"https://github\.test\.com/matthewdargan/repo/blob/[0-9a-f]{40}/README\.md#L1-L3",
        )
        .unwrap();
        assert!(re.is_match(blob_url(&args).unwrap().as_str()));
    }

    #[test]
    #[serial]
    fn test_blob_url_s_not_found() {
        let path = gix_testtools::scripted_fixture_read_only("create-readme-repo").unwrap();
        let _keep = gix_testtools::set_current_dir(path);
        let args = UrlGenerationArgs {
            link_opts: LinkOptions::Search("bar".into()),
            path: "README.md".into(),
        };
        assert!(blob_url(&args).is_err());
    }

    #[test]
    #[serial]
    fn test_blob_url_no_url() {
        let path =
            gix_testtools::scripted_fixture_read_only("create-readme-repo-no-remote").unwrap();
        let _keep = gix_testtools::set_current_dir(path);
        let args = UrlGenerationArgs {
            link_opts: LinkOptions::Empty,
            path: "README.md".into(),
        };
        let err = blob_url(&args).unwrap_err().to_string();
        assert_eq!(err, "failed to get repo URL");
    }

    #[test]
    #[serial]
    fn test_blob_url_cwd_mismatch() {
        let path = gix_testtools::scripted_fixture_read_only("create-readme-repo").unwrap();
        let args = UrlGenerationArgs {
            link_opts: LinkOptions::Empty,
            path: path.join("README.md"),
        };
        let err = blob_url(&args).unwrap_err().to_string();
        assert_eq!(err, "failed to get repo relative path");
    }

    #[test]
    #[serial]
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
    #[serial]
    fn test_gix_repo_url_no_git() {
        let path = gix_testtools::scripted_fixture_read_only("create-repo-no-git").unwrap();
        let repo = gix::open(path).unwrap();
        assert_eq!(
            gix_repo_url(&repo, gix::remote::Direction::Fetch).unwrap(),
            Some(("github.com".to_string(), "repo".to_string()))
        );
    }

    #[test]
    #[serial]
    fn test_gix_repo_url_no_remote() {
        let path = tempdir().unwrap().into_path().join("repo");
        let repo = gix::init(path).unwrap();
        assert_eq!(
            gix_repo_url(&repo, gix::remote::Direction::Fetch).unwrap(),
            None
        );
    }

    #[test]
    #[serial]
    fn test_gix_repo_url_invalid_host() {
        let path = gix_testtools::scripted_fixture_read_only("create-repo-invalid-host").unwrap();
        let repo = gix::open(path).unwrap();
        assert!(gix_repo_url(&repo, gix::remote::Direction::Fetch).is_err());
    }

    #[test]
    #[serial]
    fn test_gix_repo_url_invalid_path() {
        let path = gix_testtools::scripted_fixture_read_only("create-repo-invalid-path").unwrap();
        let repo = gix::open(path).unwrap();
        assert!(gix_repo_url(&repo, gix::remote::Direction::Fetch).is_err());
    }

    #[test]
    #[serial]
    fn test_search_lines() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"foo\nbar\nbaz\nbar").unwrap();
        assert_eq!(search_lines(file.path(), "bar").unwrap(), vec![2]);
    }

    #[test]
    #[serial]
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
    #[serial]
    fn test_search_lines_no_file() {
        let file = NamedTempFile::new().unwrap();
        assert!(search_lines(file.path(), "").is_err());
    }

    #[test]
    #[serial]
    fn test_search_lines_not_found() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"foo\nbar\nbaz\nbar").unwrap();
        assert!(search_lines(file.path(), "not found").is_err());
    }
}
