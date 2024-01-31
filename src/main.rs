pub mod blockchain;

use std::{collections::HashMap, env};

use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use blockchain::TransactionInfo;
use chrono::Duration;
use ethers::{prelude::*, utils::parse_units};
use eyre::{Context, ContextCompat, Report, Result};
use mongodb::{
	bson::{doc, oid::ObjectId},
	Client, Cursor, Database,
};
use serde::Deserialize;
use shared::{
	coin::{load_coins, Coin},
	CustomInterval,
};

use crate::blockchain::{
	account::Account,
	providers::uniswap::{UniswapProvider, UniswapTransaction},
	TradeProvider, Transaction,
};

#[derive(Deserialize, Clone, Debug)]
enum TradeSignal {
	Buy,
	Sell,
	NoAction,
}

#[derive(Deserialize, Clone, Debug)]
struct AlgorithmSignal {
	algorithm: String,
	#[serde(flatten)]
	signal: TradeSignal,
}

// have to use Duration::milliseconds due to milliseconds (and micro/nanoseconds)
// being the only way to construct a chrono::Duration in a const
pub const TRANSACTION_INTERVAL: CustomInterval =
	CustomInterval(Duration::milliseconds(5 * 60 * 1_000));

async fn get_algorithms(coin: &Coin) -> Result<HashMap<String, AlgorithmSignal>> {
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

async fn get_account_cursor(database: &Database, coin: &Coin) -> Result<Cursor<Account>> {
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

async fn run_transactions(base_coin: &Coin, coin: &Coin, database: &Database) -> Result<()> {
	let algorithms = get_algorithms(coin).await?;
	let mut accounts_cursor = get_account_cursor(database, coin).await?;

	// @TODO make this use account.provider instead of only UniswapProvider
	let provider = UniswapProvider::new()?;

	let build_transaction = |account: &Account, pair: (Coin, Coin)| -> Result<UniswapTransaction> {
		let algorithm_signal = algorithms
			.get(&account.algorithm.unwrap_or(ObjectId::default()).to_string())
			.wrap_err(format!("{:#x} has invalid algorithm", account.address))?;

		match algorithm_signal.signal {
			TradeSignal::Buy => println!("buy signal for {:#x}", account.address),
			TradeSignal::Sell => println!("sell signal for {:#x}", account.address),
			TradeSignal::NoAction => {
				return Err(Report::msg(format!(
					"no action signal for {:#x}",
					account.address
				)));
			}
		}

		let amount_input = U256::from(parse_units(10.0, pair.0.decimals)?);

		let pair_clone_1 = pair.clone();
		let pair_clone_2 = pair.clone();

		// @TODO use the provider's transaction type instead of only UniswapTransaction
		let transaction = UniswapTransaction::new(TransactionInfo {
			amount_input,
			token_in: match algorithm_signal.signal {
				TradeSignal::Buy => pair_clone_1.0,
				TradeSignal::Sell => pair_clone_1.1,
				TradeSignal::NoAction => Coin::empty(),
			},
			token_out: match algorithm_signal.signal {
				TradeSignal::Buy => pair_clone_2.1,
				TradeSignal::Sell => pair_clone_2.0,
				TradeSignal::NoAction => Coin::empty(),
			},
		});

		Ok(transaction)
	};

	while accounts_cursor.advance().await? {
		let account = accounts_cursor.deserialize_current()?;
		let pair = (base_coin.clone(), coin.clone());
		let Account { address, .. } = account;
		println!("running transactions on {:#x}", account.address);

		let transaction = build_transaction(&account, pair);
		if let Err(error) = transaction {
			eprintln!(
				"{}",
				error.wrap_err(format!("error building transaction for {:#x}", address))
			);
			continue;
		}

		let unlocked_account = account.unlock();
		if let Err(error) = unlocked_account {
			eprintln!(
				"{}",
				error.wrap_err(format!("error unlocking {:#x}", address))
			);
			continue;
		}

		let transaction = transaction.unwrap();
		println!(
			"executing transaction for {:#x}: {:#?}",
			address, transaction
		);

		match provider.swap(&transaction, &unlocked_account.unwrap()) {
			Ok(tx) => println!("transaction for {:#x} fulfilled: {:#?}", address, tx),
			Err(error) => eprintln!(
				"{}",
				error.wrap_err(format!("error executing transaction for {:#x}", address))
			),
		}
	}

	Ok(())
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
