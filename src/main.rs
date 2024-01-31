pub mod blockchain;
mod transactions;

use std::{collections::HashMap, env};

use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use chrono::Duration;
use eyre::{Context, Result};
use mongodb::{bson::doc, Client, Cursor, Database};
use serde::Deserialize;
use shared::{
	coin::{load_coins, Coin},
	CustomInterval,
};

use crate::blockchain::account::Account;
use crate::transactions::run_transactions;

#[derive(Deserialize, Clone, Debug)]
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
		println!("running transactions for {}", coin.name);
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
	dotenvy::dotenv().expect(".env should exist");

	env::var("INFURA_SECRET").expect("INFURA_SECRET should be in .env");
	let db_uri = env::var("DB_URI").expect("DB_URI should be in .env");
	let database = Client::with_uri_str(db_uri).await?.database("database");

	let _ = HttpServer::new(move || {
		let (base_coin, coins) = load_coins();
		App::new()
			.app_data(Data::new(base_coin))
			.app_data(Data::new(coins))
			.app_data(Data::new(database.clone()))
			.service(price_update)
	})
	.bind(("0.0.0.0", 80))?
	.run()
	.await;
	Ok(())
}
