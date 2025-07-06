mod client;
mod client_bd;
mod server;
mod server_bd;
mod utils;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "speedtest",
    version,
    about = "Async TCP Bandwidth Tester in Rust (with Tokio)"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "4000")]
        port: u16,

        /// Expected number of clients (terminates when all are done)
        #[arg(short, long, default_value = "4")]
        threads: usize,

        /// Block size factor (block_size = block_size_factor * 1024)
        #[arg(short = 'b', long, default_value = "64")]
        block_size_factor: usize,
    },

    /// Run the client
    Client {
        /// Server address (e.g., 127.0.0.1:4000)
        #[arg(short, long)]
        address: String,

        /// Duration in seconds to send data
        #[arg(short, long, default_value = "10")]
        duration: u64,

        /// Number of parallel connections/tasks
        #[arg(short, long, default_value = "4")]
        threads: usize,

        /// Block size factor (block_size = block_size_factor * 1024)
        #[arg(short = 'b', long, default_value = "64")]
        block_size_factor: usize,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Command::Server {
            port,
            threads,
            block_size_factor,
        } => {
            server::run_server(port, threads, block_size_factor).await;
        }
        Command::Client {
            address,
            duration,
            threads,
            block_size_factor,
        } => {
            client::run_client(address, duration, threads, block_size_factor).await;
        }
    }
}
