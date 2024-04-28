use clap::Parser;

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

fn main() {
    let args = Args::parse();
    match (args.start_line, args.end_line, &args.text) {
        (None, None, None) => {}
        (Some(_), None, None) => {}
        (Some(_), Some(_), None) => {}
        (None, None, Some(_)) => {}
        _ => eprintln!("{}", USAGE),
    }
    println!("start_line: {:?}", args.start_line);
    println!("end_line: {:?}", args.end_line);
    println!("text: {:?}", args.text);
}
