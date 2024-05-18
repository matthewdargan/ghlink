// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::str::{self, Utf8Error};

/// Returns the URL of the remote repository.
///
/// # Errors
///
/// This function will return an error if the URL is not valid UTF-8.
///
/// # Examples
///
/// ```
/// use ghlink::gix_remote_url;
///
/// let repo = gix::open(".")?;
/// let remote = repo
///     .find_default_remote(gix::remote::Direction::Fetch)
///     .unwrap()?;
/// let git_path_str = gix_remote_url(&remote)?.unwrap();
/// ```
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
