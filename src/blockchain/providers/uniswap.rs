use std::env;

use ethers::{
	prelude::Http,
	providers::Provider,
	types::{transaction::eip2718::TypedTransaction, Address},
};
use eyre::Result;
use serde::Deserialize;
use shared::coin::Coin;

use crate::blockchain::{
	account::{Account, Unlocked},
	TradeProvider, Transaction, TransactionInfo,
};

pub struct UniswapProvider {
	provider: Provider<Http>,
	quoter: Address,
	swap_router: Address,
}

impl TradeProvider for UniswapProvider {
	fn swap<T: Transaction>(
		&self,
		transaction: &T,
		account: &Account<Unlocked>,
	) -> Result<TypedTransaction> {
		todo!();
	}
	fn price(&self, base_coin: &Coin, coin: &Coin) -> Result<u32> {
		todo!();
	}
	fn combined_balance(&self, base_coin: &Coin, address: &Address) -> Result<u32> {
		todo!();
	}

	fn new() -> Result<Self> {
		let infura_secret = env::var("INFURA_SECRET").expect("INFURA_SECRET should be in .env");
		let transport_url = format!("https://mainnet.infura.io/v3/{infura_secret}");
		Provider::<Http>::try_from(transport_url)?;

		todo!();
	}
}

#[derive(Clone, Debug, Deserialize)]

pub struct UniswapTransaction {
	pub info: TransactionInfo,
	pub sqrt_price_limit: u32,
	pub amount_out_minimum: u32,
	pub fee: u16,
	pub deadline: i64,
}

impl Transaction for UniswapTransaction {
	fn verify(&self, _tx: TypedTransaction) -> Result<()> {
		todo!()
	}

	fn new(info: TransactionInfo) -> Self {
		todo!()
	}
}
