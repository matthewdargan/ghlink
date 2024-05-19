// Copyright 2024 Matthew P. Dargan. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use ghlink::{blob_url, LinkOptions, UrlGenerationArgs};
use std::env;
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process;
use std::str;

const USAGE: &str = "usage: ghlink [-l1 line1 [-l2 line2] | -s text] file";

fn parse_args() -> Result<UrlGenerationArgs, io::Error> {
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
    Ok(UrlGenerationArgs { link_opts, path })
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = match parse_args() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    println!("{}", blob_url(&cli)?);
    Ok(())
}
