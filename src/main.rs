#![feature(entry_insert)]

use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;
use project::ServerProject;
use server::Server;

mod client;
mod document;
mod project;
mod server;

#[derive(Parser)]
struct CLI {
    #[arg(short, long)]
    address: SocketAddr,
    #[arg(short, long)]
    #[clap(default_value = std::env::current_dir().expect("unable to retrieve current working directory").into_os_string())]
    project: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init()?;
    let cli = CLI::parse();
    let mut server = Server::bind(cli.address, ServerProject::from(cli.project)).await?;
    server.start().await?;
    Ok(())
}
