use std::collections::HashMap;

use eyre::{eyre, OptionExt, Result};
use mongodb::Database;
use shared::coin::Pair;
use tracing::{debug, error, instrument, warn};

use crate::{
	blockchain::{
		account::Account, chain_id, providers::uniswap::UniswapProvider, TradeProvider, TransactionInfo,
	},
	get_account_cursor, get_algorithm_signals, get_algorithms, AlgorithmSignal, TradeSignal,
};

#[instrument(err, skip(database))]
pub async fn run_transactions(database: &Database, provider: &str) -> Result<()> {
	let algorithm_id_to_name = get_algorithms(database).await?;
	let mut accounts_cursor = get_account_cursor(database, provider).await?;

	// @TODO use other providers
	let mut provider = UniswapProvider::new().await?;

	let build_transaction = |account: &Account,
	                         pair: Pair,
	                         algorithms: HashMap<String, AlgorithmSignal>|
	 -> Result<TransactionInfo> {
		let AlgorithmSignal { signal, amount, .. } = algorithms
			.get(
				algorithm_id_to_name
					.get(&account.algorithm)
					.ok_or_eyre(format!(
						"cannot find algorithm name {} for {:#x}",
						account.algorithm, account.address
					))?,
			)
			.ok_or_eyre(format!(
				"cannot find algorithm signal {} for {:#x}",
				account.algorithm, account.address
			))?;

		if let TradeSignal::NoAction = signal {
			Err(eyre!("no action signal for {:#x}", account.address))?
		}

		let transaction = TransactionInfo {
			amount: *amount,
			action: *signal,
			pair,
		};

		Ok(transaction)
	};

	while accounts_cursor.advance().await? {
		let account = match accounts_cursor.deserialize_current() {
			Ok(account) => account,
			Err(error) => {
				error!(error = ?error, "error deserializing account");
				continue;
			}
		};
		// @TODO use other pairs
		let pair = Pair::usdc_weth(Some(chain_id()));
		let algorithms = get_algorithm_signals(&pair).await?;

		let transaction = match build_transaction(&account, pair, algorithms) {
			Ok(transaction) => {
				debug!(transaction = ?transaction, account = ?account, "built transaction");
				transaction
			}
			Err(error) => {
				warn!(error = ?error, account = ?account, "error building transaction");
				continue;
			}
		};

		let _ = provider.transact(&transaction, account).await;
	}

	Ok(())
}
