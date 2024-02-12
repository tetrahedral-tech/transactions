pub mod blockchain;
mod transactions;

use std::{collections::HashMap, env};

use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use chrono::Duration;
use eyre::{Context, Result};
use mongodb::{bson::doc, Client, Cursor, Database};
use serde::{Deserialize, Serialize};
use shared::{
	coin::{load_coins, Coin},
	CustomInterval,
};
use tracing::info;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_panic::panic_hook;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::blockchain::account::Account;
use crate::transactions::run_transactions;

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub enum TradeSignal {
	Buy,
	Sell,
	NoAction,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AlgorithmSignal {
	algorithm: String,
	#[serde(flatten)]
	signal: TradeSignal,
}

// have to use Duration::milliseconds due to milliseconds (and micro/nanoseconds)
// being the only way to construct a chrono::Duration in a const
pub const TRANSACTION_INTERVAL: CustomInterval =
	CustomInterval(Duration::milliseconds(5 * 60 * 1_000));

pub async fn get_algorithms(coin: &Coin) -> Result<HashMap<String, AlgorithmSignal>> {
	Ok(
		reqwest::get(format!(
			"{}/signals?coin={}",
			env::var("ALGORITHM_SERVER_URI").expect("ALGORITHM_SERVER_URI should be in .env"),
			coin.name
		))
		.await?
		.json::<Vec<AlgorithmSignal>>()
		.await?
		.into_iter()
		.map(|x| (x.algorithm.clone(), x))
		.collect(),
	)
}

pub async fn get_account_cursor(database: &Database, coin: &Coin) -> Result<Cursor<Account>> {
	// @TODO sort by user subscription level
	let opts = None; // FindOptions::builder().sort(doc! {}).build();
	let cursor = database
		.collection::<Account>("bots")
		.find(
			doc! {
				"status": {
					"name": "running"
				},
				"coin": coin.name.as_str()
			},
			opts,
		)
		.await?;

	Ok(cursor)
}

#[get("/price_update")]
async fn price_update(
	base_coin: Data<Coin>,
	coins: Data<Vec<Coin>>,
	database: Data<Database>,
) -> impl Responder {
	let base_coin = base_coin.as_ref();
	let coins = coins.as_ref();
	let database = database.as_ref();

	for coin in coins {
		match run_transactions(base_coin, coin, database)
			.await
			.wrap_err("error with transactions for coin {}")
		{
			Ok(_) => println!("transactions for {} completed", coin.name),
			Err(error) => eprintln!("{}", error),
		}
	}

	HttpResponse::Ok().body("")
}

#[tokio::main]
async fn main() -> Result<()> {
	let subscriber = Registry::default()
		.with(JsonStorageLayer)
		.with(BunyanFormattingLayer::new(
			"price-collector".into(),
			std::fs::File::create("server.log")?,
		))
		.with(BunyanFormattingLayer::new(
			"price-collector".into(),
			std::io::stdout,
		));

	tracing::subscriber::set_global_default(subscriber).unwrap();
	std::panic::set_hook(Box::new(panic_hook));

	dotenvy::dotenv().expect(".env should exist");

	env::var("INFURA_SECRET").expect("INFURA_SECRET should be in .env");
	let db_uri = env::var("DB_URI").expect("DB_URI should be in .env");

	let database = Client::with_uri_str(db_uri).await?.database("database");

	let bind_to = ("0.0.0.0", 80);
	let server_future = HttpServer::new(move || {
		let (base_coin, coins) = load_coins();
		App::new()
			.app_data(Data::new(base_coin))
			.app_data(Data::new(coins))
			.app_data(Data::new(database.clone()))
			.service(price_update)
	})
	.bind(bind_to)
	.expect(format!("{}:{} should be available to bind to", bind_to.0, bind_to.1).as_str())
	.run();

	info!(host = bind_to.0, port = bind_to.1, "running server");

	server_future.await?;

	Ok(())
}
