use eyre::{eyre, ContextCompat, Result};
use mongodb::Database;
use shared::coin::Coin;
use tracing::{debug, error, info, instrument};

use crate::{
	blockchain::{account::Account, providers::uniswap::UniswapPool, TransactionInfo},
	get_account_cursor, get_algorithms, TradeSignal,
};

#[instrument(err, skip(database))]
pub async fn run_transactions(base_coin: &Coin, coin: &Coin, database: &Database) -> Result<()> {
	let algorithms = get_algorithms(coin).await?;
	let mut accounts_cursor = get_account_cursor(database, coin).await?;

	// @TODO make this use account.provider instead of only UniswapPool
	let mut pool = UniswapPool::new()?;

	let build_transaction = |account: &Account, pair: (Coin, Coin)| -> Result<TransactionInfo> {
		let algorithm_signal = algorithms
			.get(&account.algorithm.to_string())
			.wrap_err(format!(
				"cannot find algorithm {} for {:#x}",
				account.algorithm, account.address
			))?;

		if let TradeSignal::NoAction = algorithm_signal.signal {
			Err(eyre!("no action signal for {:#x}", account.address))?
		}

		let pair_clone = pair.clone();

		let transaction = TransactionInfo {
			amount: 10.0,
			action: algorithm_signal.signal,
			token_base: pair_clone.0,
			token_other: pair_clone.1,
		};

		Ok(transaction)
	};

	while accounts_cursor.advance().await? {
		let account = accounts_cursor.deserialize_current()?;
		if account.status.name != "running" {
			debug!(account = ?account, "skipping not running account");
			continue;
		}

		let pair = (base_coin.clone(), coin.clone());

		let transaction = match build_transaction(&account, pair) {
			Ok(transaction) => {
				debug!(transaction = ?transaction, account = ?account, "built transaction");
				transaction
			}
			Err(error) => {
				error!(error = ?error, account = ?account, "error building transaction");
				continue;
			}
		};

		info!(transaction = ?transaction, account = ?account, "pushing transaction to pool");

		let _ = pool.push(&transaction);
	}

	Ok(())
}
