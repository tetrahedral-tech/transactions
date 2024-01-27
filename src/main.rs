mod blockchain;

use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use chrono::Duration;
use ethers::prelude::*;
use eyre::Result;
use mongodb::{
	bson::{doc, oid::ObjectId},
	Client, Cursor, Database,
};
use serde::Deserialize;
use shared::{
	coin::{load_coins, Coin},
	CustomInterval,
};

use std::{collections::HashMap, env, sync::Arc};

#[derive(Deserialize, Clone, Debug)]
enum TradeSignal {
	Buy,
	Sell,
}

#[derive(Deserialize, Clone, Debug)]
struct AlgorithmSignal {
	algorithm: String,
	signal: TradeSignal,
	coin: Coin,
	amount: u32,
}

#[derive(Deserialize, Clone, Debug)]
struct Account {
	address: String,
	algorithm: ObjectId,
	encrypted_private_key: String,
}

impl Account {
	pub fn private_key(&self) -> String {
		todo!("implement private key decryption");
	}
}

// have to use Duration::milliseconds due to milliseconds (and micro/nanoseconds)
// being the only way to construct a chrono::Duration in a const
pub const TRANSACTION_INTERVAL: CustomInterval =
	CustomInterval(Duration::milliseconds(5 * 60 * 1_000));

async fn get_algorithms() -> Result<HashMap<String, AlgorithmSignal>> {
	Ok(
		reqwest::get(format!(
			"{}/algorithms",
			env::var("ALGORITHM_SERVER_URI").expect("ALGORITHM_SERVER_URI should be in .env")
		))
		.await?
		.json::<Vec<AlgorithmSignal>>()
		.await?
		.into_iter()
		.map(|x| (x.algorithm.clone(), x))
		.collect(),
	)
}

async fn get_account_cursor(database: &Database) -> Result<Cursor<Account>> {
	// @TODO sort by user subscription level
	let opts = None; // FindOptions::builder().sort(doc! {}).build();
	let cursor = database
		.collection::<Account>("bots")
		.find(
			doc! {
				"status": {
					"name": "running"
				}
			},
			opts,
		)
		.await?;

	Ok(cursor)
}

async fn run_transactions(
	base_coin: Data<Coin>,
	provider: Data<Provider<Http>>,
	database: Data<Database>,
) -> Result<()> {
	let algorithms = get_algorithms().await?;
	let mut accounts_cursor = get_account_cursor(database.get_ref()).await?;

	while accounts_cursor.advance().await? {
		let account = accounts_cursor.deserialize_current()?;
		let algorithm = algorithms.get(&account.algorithm.to_string());

		if algorithm.is_none() {
			continue;
		}

		let algorithm = algorithm.unwrap();
	}

	Ok(())
}

#[get("/price_update")]
async fn price_update(
	base_coin: Data<Coin>,
	provider: Data<Provider<Http>>,
	database: Data<Database>,
) -> impl Responder {
	match run_transactions(base_coin, provider, database).await {
		Ok(_) => HttpResponse::Ok().body(""),
		Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	dotenvy::dotenv().expect(".env should exist");

	let db_uri = env::var("DB_URI").expect("DB_URI should be in .env");
	let infura_secret = env::var("INFURA_SECRET").expect("INFURA_SECRET should be in .env");
	let transport_url = format!("https://mainnet.infura.io/v3/{infura_secret}");

	let database = Client::with_uri_str(db_uri).await?.database("database");
	let web3_provider = Arc::new(Provider::<Http>::try_from(transport_url)?);

	let _ = HttpServer::new(move || {
		let (base_coin, _) = load_coins();
		App::new()
			.app_data(Data::new(base_coin))
			.app_data(Data::new(database.clone()))
			.app_data(Data::new(web3_provider.clone()))
			.service(price_update)
	})
	.bind(("0.0.0.0", 80))?
	.run()
	.await;
	Ok(())
}
