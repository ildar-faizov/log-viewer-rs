use clap::Parser;
use log::LevelFilter;


#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    #[arg(short, long)]
    pub file: Option<String>,

    #[arg(short='L', long="loglevel", help="One of \"OFF\", \"ERROR\", \"WARN\", \"INFO\", \"DEBUG\", \"TRACE\". Case-insensitive.")]
    pub log_level: Option<LevelFilter>,

    #[arg(short, long, help = "Port for puffin profiler to connect to. Puffin viewer expects 8585 by default. Profiler is disabled, if not specified.")]
    pub profiler_port: Option<u16>,
}