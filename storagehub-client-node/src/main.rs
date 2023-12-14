//! Storagehub client node.

use clap::{Parser, ValueEnum};
use options::Options;
use std::{env, error::Error};
use tokio::spawn;
use tracing::level_filters::LevelFilter;
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
	let filter = if env::var(EnvFilter::DEFAULT_ENV).is_ok() {
		EnvFilter::from_default_env()
	} else {
		EnvFilter::default().add_directive(LevelFilter::INFO.into())
	};
	tracing_subscriber::fmt().with_env_filter(filter).init();

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
