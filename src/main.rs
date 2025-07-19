mod client;
mod file;
mod server;
mod utils;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ValueEnum)]
#[clap(rename_all = "lowercase")]
enum Direction {
    Upload,
    Download,
    Bidirectional,
    Quit,
}

#[derive(Parser)]
#[command(name = "speedtest", version, about = "Async TCP Bandwidth Tester in Rust (with Tokio)")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Server {
        #[arg(short, long, default_value = "4000")]
        port: u16,

        #[arg(short = 'b', long, default_value = "64")]
        block_size_kb: usize,

        #[arg(short = 'd', long, default_value = "10")]
        duration_secs: u64,
    },
    Client {
        #[arg(short, long)]
        address: String,

        #[arg(short = 't', long, default_value = "4")]
        threads: usize,

        #[arg(short = 'b', long, default_value = "64")]
        block_size_kb: usize,

        #[arg(short = 'd', long, default_value = "10")]
        duration_secs: u64,

        #[arg(long, value_enum, default_value = "upload")]
        direction: Direction,
    },
    Loop {
        #[arg(short, long)]
        address: String,

        #[arg(short = 't', long, default_value = "4")]
        threads: usize,

        #[arg(short = 'b', long, default_value = "64")]
        block_size_kb: usize,

        #[arg(short = 'd', long, default_value = "10")]
        duration_secs: u64,

        #[arg(short = 'p', long)]
        path: String,

        #[arg(short = 's', long, default_value = "100", help = "Maximum size of file to write and read in MB (default: 100 MB)")]
        file_size_mb: usize,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command.unwrap_or(
        /*
            Command::Server {
            port: 4000,
            block_size_kb: 64,
            duration_secs: 10,
        }*/
        Command::Loop {
            address: "127.0.0.1:4000".to_string(),
            threads: 4,
            block_size_kb: 100,
            duration_secs: 10,
            path: "./testfile.txt".to_string(),
            file_size_mb: 10 * 1024 * 1024,
        },
    ) {
        Command::Server { port, block_size_kb, duration_secs } => {
            server::run_server(port, block_size_kb, duration_secs).await;
        }
        Command::Client {
            address,
            threads,
            block_size_kb,
            duration_secs,
            direction,
        } => {
            client::run_client(address, threads, block_size_kb, duration_secs, direction).await;
        }
        Command::Loop {
            address,
            threads,
            block_size_kb,
            duration_secs,
            path,
            file_size_mb,
        } => {
            client::run_client_loop(address, threads, block_size_kb, duration_secs, &path, file_size_mb).await;
        }
    }
}
