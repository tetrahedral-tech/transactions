pub mod account;
pub mod providers;

use async_trait::async_trait;
use eyre::Result;
use phf::{phf_map, Map};
use serde::{Deserialize, Serialize};
use shared::coin::Pair;

use self::account::{Account, Locked};
use crate::TradeSignal;

static CHAIN_NAME_TO_ID: Map<&'static str, u16> = phf_map! {
	"goerli" => 0x5
};
const CURRENT_CHAIN_NAME: &'static str = "goerli";

pub fn chain_id() -> u16 {
	*CHAIN_NAME_TO_ID
		.get(CURRENT_CHAIN_NAME)
		.expect("current chain name not in chain to name map")
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionInfo {
	pub amount: f64,
	pub action: TradeSignal,
	pub pair: Pair,
}

#[async_trait]
pub trait TradeProvider {
	async fn transact(
		&mut self,
		transaction: &TransactionInfo,
		account: Account<Locked>,
	) -> Result<()>;
	async fn new() -> Result<Self>
	where
		Self: std::marker::Sized;
}
