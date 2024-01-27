use ethers::types::{transaction::eip2718::TypedTransaction, Address};
use eyre::Result;
use shared::coin::Coin;

pub trait VerifyTransaction {
	fn verify(&self, tx: TypedTransaction) -> Result<()>;
}

pub trait TradeProvider {
	fn swap(&self, transaction: Transaction) -> Result<TypedTransaction>;
	fn get_price(&self, base_coin: Coin, coin: Coin) -> Result<u32>;
	fn get_net_worth(&self, base_coin: Coin, address: Address) -> Result<u32>;
	fn name(&self) -> String;
}

pub struct Transaction {
	address: Address,
	amount_input: u32,
	token_in: Coin,
	token_out: Coin,
}

pub struct UniswapTransaction {
	transaction: Transaction,
}

impl VerifyTransaction for UniswapTransaction {
	fn verify(&self, _tx: TypedTransaction) -> Result<()> {
		Ok(()) // @TODO implement transaction verification
	}
}

pub struct Provider {
	name: String,
}
