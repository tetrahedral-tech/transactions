pub mod account;
pub mod providers;

use ethers::types::{transaction::eip2718::TypedTransaction, Address, U256};
use eyre::Result;
use serde::Deserialize;
use shared::coin::Coin;

use crate::Account;

use self::account::Unlocked;

#[derive(Clone, Debug, Deserialize)]
pub struct TransactionInfo {
	pub amount_input: U256,
	pub token_in: Coin,
	pub token_out: Coin,
}
pub trait Transaction {
	fn verify(&self, tx: TypedTransaction) -> Result<()>;
	fn new(info: TransactionInfo) -> Self;
}

pub trait TradeProvider {
	fn swap<T: Transaction>(
		&self,
		transaction: &T,
		account: &Account<Unlocked>,
	) -> Result<TypedTransaction>;
	fn price(&self, base_coin: &Coin, coin: &Coin) -> Result<u32>;
	fn combined_balance(&self, base_coin: &Coin, address: &Address) -> Result<u32>;
	fn new() -> Result<Self>
	where
		Self: std::marker::Sized;
}
