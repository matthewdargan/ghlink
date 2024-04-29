use std::error::Error;

#[derive(Debug)]
struct Cli {
    l1: Option<usize>,
    l2: Option<usize>,
    search: Option<String>,
    path: Option<std::path::PathBuf>,
}

const USAGE: &str = "usage: ghlink [-l1 line1 [-l2 line2] | -s text] path";

fn parse_args() -> Option<Cli> {
    let mut args = std::env::args().skip(1);
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
            _ => cli.path = Some(std::path::PathBuf::from(arg)),
        }
    }
    if cli.l1.is_none() && cli.l2.is_some() {
        return None;
    }
    cli.path.as_ref()?;
    Some(cli)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = match parse_args() {
        Some(cli) => cli,
        None => {
            eprintln!("{}", USAGE);
            return Ok(());
        }
    };
    let mut path = std::fs::canonicalize(cli.path.as_ref().unwrap())?;
    if path.is_file() {
        path.pop();
    }
    let repo = gix::discover(&path)?;
    let remote = repo
        .find_default_remote(gix::remote::Direction::Fetch)
        .unwrap()?;
    let git_path = &remote
        .url(gix::remote::Direction::Fetch)
        .unwrap()
        .path
        .strip_suffix(".git".as_bytes())
        .unwrap();
    let git_path_str = std::str::from_utf8(git_path)?;
    let commit = repo.rev_parse_single("HEAD")?;
    let prefix = repo.prefix()?;
    let prefix_str = prefix.as_ref().unwrap();
    let path_ref = cli.path.as_ref().unwrap();
    let joined_path = prefix_str.join(path_ref);
    let rel_path = joined_path.to_str().unwrap();
    let mut url = format!(
        "https://github.com/{}/blob/{}/{}",
        git_path_str, commit, rel_path
    );
    if cli.l1.is_some() {
        url.push_str(&format!("#L{}", cli.l1.unwrap()));
    }
    if cli.l2.is_some() {
        url.push_str(&format!("-L{}", cli.l2.unwrap()));
    }
    println!("{:?}", url);
    Ok(())
}
