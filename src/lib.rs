use std::str::{self, Utf8Error};

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
