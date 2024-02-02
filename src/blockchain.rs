pub mod account;
pub mod providers;

use async_trait::async_trait;
use ethers::types::{transaction::eip2718::TypedTransaction, Address};
use eyre::Result;
use serde::Deserialize;
use shared::coin::Coin;

use crate::TradeSignal;

#[derive(Clone, Debug, Deserialize)]
pub struct TransactionInfo {
	pub amount_input: f64,
	pub token_base: Coin,
	pub token_other: Coin,
	pub trade_type: TradeSignal,
}

pub trait Transaction {
	fn verify(&self, tx: TypedTransaction) -> Result<()>;
	fn new(info: TransactionInfo) -> Self;
}

#[async_trait]
pub trait TradeProvider {
	async fn price(&self, base_coin: &Coin, coin: &Coin) -> Result<u32>;
	async fn combined_balance(&self, base_coin: &Coin, address: &Address) -> Result<u32>;
	fn new() -> Result<Self>
	where
		Self: std::marker::Sized;
}
