use clap::Parser;
use std::path::PathBuf;
use bootiful_plymouth_discovery::collect_all;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    #[arg(short, long, default_value_t = 2)]
    indent: usize,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    let config = collect_all()?;
    let json = serde_json::to_string_pretty(&config)?;
    
    if let Some(path) = cli.output {
        std::fs::write(path, json)?;
    } else {
        println!("{}", json);
    }
    
    Ok(())
}
