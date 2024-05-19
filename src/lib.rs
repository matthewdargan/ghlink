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
