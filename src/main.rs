use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
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

fn main() {
    let args = Args::parse();
    println!("start_line: {:?}", args.start_line);
    println!("end_line: {:?}", args.end_line);
    println!("text: {:?}", args.text);
}
