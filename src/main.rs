use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Sets a path to link
    path: std::path::PathBuf,
    /// Print link to start line number
    #[arg(short, long)]
    start_line: Option<usize>,
    /// Print link to end line number
    #[arg(short, long)]
    end_line: Option<usize>,
    /// Print link to matching text
    #[arg(short, long)]
    text: Option<String>,
}

const USAGE: &str = "usage: ghlink [-s start-line [-e end-line] | -t text] path";

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match (args.start_line, args.end_line, &args.text) {
        (None, None, None) => {}
        (Some(_), None, None) => {}
        (Some(_), Some(_), None) => {}
        (None, None, Some(_)) => {}
        _ => eprintln!("{}", USAGE),
    }
    let mut path = std::fs::canonicalize(&args.path)?;
    if path.is_file() {
        path.pop();
    }
    let repo = gix::discover(&path)?;
    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .unwrap()?;
    let url = &remote.url(gix::remote::Direction::Fetch).unwrap().path;
    let commit = repo.rev_parse_single("HEAD")?;
    println!("{:?}", path);
    println!("{:?}", url);
    println!("{:?}", commit);
    Ok(())
}
