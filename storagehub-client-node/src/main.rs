use clap::{Parser, ValueEnum};
use options::Options;
use std::error::Error;
use tokio::spawn;
use tracing_subscriber::EnvFilter;

mod lightclient;
mod options;
mod p2p;

#[derive(ValueEnum, Clone, Debug)]
pub enum Role {
	User,
	BspProvider,
	MspProvider,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

	let opts: Options = Options::parse();

	let service = p2p::service::Service::new(
		opts.run_as.clone(),
		opts.libp2p_options.port,
		opts.upload_path,
	)?;

	let sender = service.command_sender();

	spawn(lightclient::client::Client::run(
		opts.run_as,
		opts.light_client_options,
		sender,
		opts.download_path,
	));

	service.run().await?;

	Ok(())
}
