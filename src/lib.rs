// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::str::{self, Utf8Error};

/// Returns the URL of the remote repository.
///
/// # Errors
///
/// This function will return an error if the URL is not valid UTF-8.
pub fn gix_remote_url(remote: &gix::Remote) -> Result<Option<String>, Utf8Error> {
    let git_path = match remote.url(gix::remote::Direction::Fetch) {
        Some(url) => match url.path.strip_suffix(b".git") {
            Some(path) => path,
            None => &url.path,
        },
        None => return Ok(None),
    };
    Ok(Some(str::from_utf8(git_path)?.to_string()))
}

/// Search for `text` in the file at `path` and return the line numbers.
///
/// # Errors
///
/// This function will return an error if the file cannot be read or if no lines contain `text`.
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
