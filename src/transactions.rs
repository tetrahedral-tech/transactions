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

		let pair_clone = pair.clone();

		// @TODO use the provider's transaction type instead of only UniswapTransaction
		let transaction = UniswapTransaction::new(TransactionInfo {
			amount_input: 10.0,
			token_base: pair_clone.0,
			token_other: pair_clone.1,
			trade_type: algorithm_signal.signal,
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

		match provider
			.swap(&transaction, &unlocked_account.unwrap())
			.await
		{
			Ok(tx) => println!("transaction for {:#x} fulfilled: {:#?}", address, tx),
			Err(error) => eprintln!(
				"{}",
				error.wrap_err(format!("error executing transaction for {:#x}", address))
			),
		}
	}

	Ok(())
}
