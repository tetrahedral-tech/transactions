pub mod blockchain;
mod transactions;

use std::fs;
use std::{collections::HashMap, env};

use actix_web::{get, web, web::Data, App, HttpResponse, HttpServer, Responder};
use chrono::Duration;
use eyre::{eyre, Context, Result};
use futures_util::TryStreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::{bson::doc, Client, Cursor, Database};
use serde::{Deserialize, Serialize};
use shared::{coin::Pair, CustomInterval};
use tracing::{error, info};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_panic::panic_hook;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::blockchain::account::Account;
use crate::transactions::run_transactions;

#[derive(Deserialize, Serialize, Clone, Debug, Copy)]
pub enum TradeSignal {
	#[serde(rename = "buy")]
	Buy,
	#[serde(rename = "sell")]
	Sell,
	#[serde(rename = "no_action")]
	NoAction,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AlgorithmSignal {
	algorithm: String,
	signal: TradeSignal,
	amount: f64,
}

// have to use Duration::milliseconds due to milliseconds (and micro/nanoseconds)
// being the only way to construct a chrono::Duration in a const
pub const TRANSACTION_INTERVAL: CustomInterval =
	CustomInterval(Duration::milliseconds(5 * 60 * 1_000));

pub async fn get_algorithm_signals(pair: &Pair) -> Result<HashMap<String, AlgorithmSignal>> {
	let url = format!(
		// @TODO uncomment these once pair is used everywhere in algo server
		// "{}/signals?pair={},{}&interval={}"
		"{}/signals?coin={}&interval={}",
		env::var("ALGORITHM_SERVER_URI").expect("ALGORITHM_SERVER_URI should be in .env"),
		// pair.0.name,
		pair.1.name,
		5 //@TODO use other intervals
	);

	let response = reqwest::get(url).await?;

	Ok(
		response
			.json::<Vec<AlgorithmSignal>>()
			.await
			.wrap_err(eyre!("could not parse algorithm server response"))?
			.into_iter()
			.map(|x| (x.algorithm.clone(), x))
			.collect(),
	)
}

#[derive(Deserialize)]
struct Algorithm {
	_id: ObjectId,
	name: String,
}

pub async fn get_algorithms(database: &Database) -> Result<HashMap<ObjectId, String>> {
	let cursor = database
		.collection::<Algorithm>("algorithms")
		.find(doc! {}, None)
		.await?;

	let algorithms: Vec<Algorithm> = cursor.try_collect().await?;
	let algorithms: HashMap<ObjectId, String> = algorithms
		.into_iter()
		.map(|Algorithm { _id, name }| (_id, name))
		.collect();
	Ok(algorithms)
}

pub async fn get_account_cursor(database: &Database, provider: &str) -> Result<Cursor<Account>> {
	// @TODO sort by user subscription level
	let opts = None; // FindOptions::builder().sort(doc! { subLevel }).build();
	let cursor = database
		.collection::<Account>("bots")
		.find(
			doc! {
				"status.name": "running",
				"provider": provider
			},
			opts,
		)
		.await?;

	Ok(cursor)
}

#[get("/price_update")]
async fn price_update(database: Data<Database>, timestamp: web::Query<i64>) -> impl Responder {
	let database = database.as_ref();

	// @TODO use other providers
	match run_transactions(database, *timestamp, "uniswap").await {
		Ok(_) => info!("transactions completed"),
		Err(error) => error!(error = ?error, "error with running transactions"),
	}

	HttpResponse::Ok().body("")
}

#[tokio::main]
async fn main() -> Result<()> {
	let subscriber = Registry::default()
		.with(JsonStorageLayer)
		.with(BunyanFormattingLayer::new(
			"transactions".into(),
			std::fs::File::options()
				.append(true)
				.create(true)
				.open("transactions.log")?,
		))
		.with(BunyanFormattingLayer::new(
			"transactions".into(),
			std::io::stdout,
		));

	tracing::subscriber::set_global_default(subscriber)
		.expect("setting global default subscriber should succeed");

	let prev_hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |panic_info| {
		panic_hook(panic_info);
		prev_hook(panic_info);
	}));

	dotenvy::dotenv().expect(".env should exist");

	fs::metadata("transaction-router/").expect("transaction-router/ should exist");
	env::var("ROUTER_ADDRESS").expect("ROUTER_ADDRESS should be in .env");
	env::var("INFURA_SECRET").expect("INFURA_SECRET should be in .env");
	let db_uri = env::var("DB_URI").expect("DB_URI should be in .env");

	let database = Client::with_uri_str(db_uri).await?.database("database");

	let bind = ("0.0.0.0", 80);
	let server = HttpServer::new(move || {
		App::new()
			.app_data(Data::new(database.clone()))
			.service(price_update)
	})
	.bind(bind)
	.expect(format!("{}:{} should be available to bind to", bind.0, bind.1).as_str())
	.run();

	info!(host = bind.0, port = bind.1, "running server");
	server.await?;

	Ok(())
}
