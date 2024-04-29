use std::error::Error;
use std::io::BufRead;

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
    if cli.search.is_some() && (cli.l1.is_some() || cli.l2.is_some()) {
        return None;
    }
    cli.path.as_ref()?;
    Some(cli)
}

fn search_lines(path: &std::path::Path, text: &str) -> std::io::Result<Vec<usize>> {
    // TODO: text.split('\n') and iterate each split text found in line
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let mut line_nums = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        if line?.contains(text) {
            line_nums.push(i + 1);
        }
    }
    if line_nums.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("file {} does not contain string {}", path.display(), text),
        ));
    }
    Ok(line_nums)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = match parse_args() {
        Some(cli) => cli,
        None => {
            eprintln!("{}", USAGE);
            return Ok(());
        }
    };
    let path = cli.path.unwrap();
    let mut abs_path = std::fs::canonicalize(&path)?;
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
    let git_path_str = std::str::from_utf8(git_path)?;
    let commit = repo.rev_parse_single("HEAD")?;
    let prefix = repo.prefix()?;
    let joined = prefix.unwrap().join(&path);
    let rel_path = joined.to_str().unwrap();
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
    if cli.search.is_some() {
        let line_nums = search_lines(path.as_path(), &cli.search.unwrap());
        println!("{:?}", line_nums);
    }
    println!("{:?}", url);
    Ok(())
}
