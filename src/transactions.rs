use ethers::{prelude::*, utils::parse_units};
use eyre::{ContextCompat, Report, Result};
use mongodb::{bson::oid::ObjectId, Database};
use shared::coin::Coin;

use crate::{
	blockchain::{
		account::Account,
		providers::uniswap::{UniswapProvider, UniswapTransaction},
		TradeProvider, Transaction, TransactionInfo,
	},
	get_account_cursor, get_algorithms, TradeSignal,
};

pub async fn run_transactions(base_coin: &Coin, coin: &Coin, database: &Database) -> Result<()> {
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
