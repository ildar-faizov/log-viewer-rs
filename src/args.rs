use clap::Parser;
use log::LevelFilter;


#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long)]
    pub file: Option<String>,

    #[arg(short='L', long="loglevel", help="One of \"OFF\", \"ERROR\", \"WARN\", \"INFO\", \"DEBUG\", \"TRACE\". Case-insensitive.")]
    pub log_level: Option<LevelFilter>,
}