pub mod account;
pub mod providers;

use serde::{Deserialize, Serialize};
use shared::coin::Coin;

use crate::TradeSignal;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionInfo {
	pub amount: f64,
	pub action: TradeSignal,
	pub token_base: Coin,
	pub token_other: Coin,
}
