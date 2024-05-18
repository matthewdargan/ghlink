// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use ghlink::{gix_remote_url, search_lines};
use std::env;
use std::error::Error;
use std::fs::{self};
use std::io::{self};
use std::path::PathBuf;
use std::process;
use std::str;

#[derive(Debug)]
struct Cli {
    link_opts: LinkOptions,
    path: PathBuf,
}

#[derive(Debug)]
enum LinkOptions {
    Lines(usize, Option<usize>),
    Search(String),
    Empty,
}

const USAGE: &str = "usage: ghlink [-l1 line1 [-l2 line2] | -s text] file";

fn parse_args() -> Result<Cli, io::Error> {
    let mut args = env::args().skip(1);
    let mut l1 = None;
    let mut l2 = None;
    let mut search = None;
    let mut path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-l1" => {
                l1 = Some(
                    args.next()
                        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, USAGE))?
                        .parse::<usize>()
                        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, USAGE))?,
                );
            }
            "-l2" => {
                l2 = Some(
                    args.next()
                        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, USAGE))?
                        .parse::<usize>()
                        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, USAGE))?,
                );
            }
            "-s" => search = args.next(),
            _ => path = Some(PathBuf::from(arg)),
        }
    }
    let link_opts = match (l1, l2, search) {
        (Some(l1), _, None) => LinkOptions::Lines(l1, l2),
        (None, None, Some(search)) => LinkOptions::Search(search),
        (None, None, None) => LinkOptions::Empty,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE)),
    };
    let Some(path) = path else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, USAGE));
    };
    Ok(Cli { link_opts, path })
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = match parse_args() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    let mut url = {
        let mut abs_path = fs::canonicalize(&cli.path)?;
        if abs_path.is_file() {
            abs_path.pop();
        }
        let repo = gix::discover(&abs_path)?;
        let remote = repo
            .find_default_remote(gix::remote::Direction::Fetch)
            .unwrap()?;
        let git_path_str = gix_remote_url(&remote)?.unwrap();
        let commit = repo.rev_parse_single("HEAD")?;
        let prefix = repo.prefix()?;
        let joined = prefix.unwrap().join(&cli.path);
        let rel_path = joined.to_str().unwrap();
        format!("https://github.com/{git_path_str}/blob/{commit}/{rel_path}")
    };
    match cli.link_opts {
        LinkOptions::Lines(l1, l2) => {
            url.push_str(&format!("#L{l1}"));
            if let Some(l2) = l2 {
                url.push_str(&format!("-L{l2}"));
            }
        }
        LinkOptions::Search(mut search) => {
            if search == "-" {
                search = io::read_to_string(io::stdin())?;
            }
            let line_nums = search_lines(cli.path.as_path(), &search)?;
            url.push_str(&format!("#L{}", line_nums.first().unwrap()));
            if line_nums.len() > 1 {
                url.push_str(&format!("-L{}", line_nums.last().unwrap()));
            }
        }
        LinkOptions::Empty => {}
    }
    println!("{url}");
    Ok(())
}
