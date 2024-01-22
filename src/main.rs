use std::{env, sync::Arc};

use chrono::Duration;
use clokwerk::AsyncScheduler;
use ethers::prelude::*;
use eyre::Result;
use shared::{coin::load_coins, CustomInterval};

// have to use Duration::milliseconds due to milliseconds (and micro/nanoseconds)
// being the only way to construct a chrono::Duration in a const
pub const TRANSACTION_INTERVAL: CustomInterval =
	CustomInterval(Duration::milliseconds(5 * 60 * 1_000));

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().expect(".env should exist");

	let infura_secret = env::var("INFURA_SECRET").expect("INFURA_SECRET should be in .env");
	let transport_url = format!("https://mainnet.infura.io/v3/{infura_secret}");
	let web3_provider = Arc::new(Provider::<Http>::try_from(transport_url)?);

	let mut scheduler = AsyncScheduler::new();
	let (base_coin, coins) = load_coins();

	scheduler
		.every(TRANSACTION_INTERVAL.interval())
		.run(move || {
			println!("collecting prices");
			let provider_clone = web3_provider.clone();
			let base_coin_clone = base_coin.clone();
			let coins_clone = coins.clone();

			async move {}
		});

	loop {
		scheduler.run_pending().await;
		tokio::time::sleep(
			Duration::milliseconds(10)
				.to_std()
				.expect("10ms sleep could not parse to std"),
		)
		.await;
	}
}
