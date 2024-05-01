// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str;

#[derive(Debug)]
struct Cli {
    l1: Option<usize>,
    l2: Option<usize>,
    search: Option<String>,
    path: Option<PathBuf>,
}

fn parse_args() -> Option<Cli> {
    let mut args = env::args().skip(1);
    let mut cli = Cli {
        l1: None,
        l2: None,
        search: None,
        path: None,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-l1" => {
                cli.l1 = Some(args.next()?.parse::<usize>().ok()?);
            }
            "-l2" => {
                cli.l2 = Some(args.next()?.parse::<usize>().ok()?);
            }
            "-s" => cli.search = args.next(),
            _ => cli.path = Some(PathBuf::from(arg)),
        }
    }
    match (cli.l1.is_some(), cli.l2.is_some(), cli.search.is_some()) {
        (false, true, _) | (true, _, true) => None,
        _ => {
            cli.path.as_ref()?;
            Some(cli)
        }
    }
}

fn search_lines(path: &Path, text: &str) -> io::Result<Vec<usize>> {
    let reader = BufReader::new(File::open(path)?);
    let mut text_lines = text.lines().peekable();
    let mut line_nums = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        if let Some(text_line) = text_lines.peek() {
            if line?.contains(text_line) {
                line_nums.push(i + 1);
                text_lines.next();
            }
        }
    }
    if line_nums.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("file {} does not contain string {}", path.display(), text),
        ));
    }
    Ok(line_nums)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = match parse_args() {
        Some(cli) => cli,
        None => {
            eprintln!("usage: ghlink [-l1 line1 [-l2 line2] | -s text] file");
            return Ok(());
        }
    };
    let path = cli.path.unwrap();
    let mut abs_path = fs::canonicalize(&path)?;
    if abs_path.is_file() {
        abs_path.pop();
    }
    let repo = gix::discover(&abs_path)?;
    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .unwrap()?;
    let git_path = &remote
        .url(gix::remote::Direction::Fetch)
        .unwrap()
        .path
        .strip_suffix(".git".as_bytes())
        .unwrap();
    let git_path_str = str::from_utf8(git_path)?;
    let commit = repo.rev_parse_single("HEAD")?;
    let prefix = repo.prefix()?;
    let joined = prefix.unwrap().join(&path);
    let rel_path = joined.to_str().unwrap();
    let mut url = format!(
        "https://github.com/{}/blob/{}/{}",
        git_path_str, commit, rel_path
    );
    if let Some(l1) = cli.l1 {
        url.push_str(&format!("#L{}", l1));
    }
    if let Some(l2) = cli.l2 {
        url.push_str(&format!("-L{}", l2));
    }
    if let Some(mut search) = cli.search {
        if search == "-" {
            search = io::read_to_string(io::stdin())?;
        }
        let line_nums = search_lines(path.as_path(), &search)?;
        url.push_str(&format!("#L{}", line_nums.first().unwrap()));
        if line_nums.len() > 1 {
            url.push_str(&format!("-L{}", line_nums.last().unwrap()));
        }
    }
    println!("{}", url);
    Ok(())
}
