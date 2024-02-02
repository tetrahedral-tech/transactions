use std::{env, str::FromStr, sync::Arc};

use async_trait::async_trait;
use chrono::{Duration, Utc};
use ethers::{
	prelude::Http,
	providers::Provider,
	types::{transaction::eip2718::TypedTransaction, Address, TransactionReceipt, U256},
	utils::parse_units,
};
use eyre::{eyre, ContextCompat};
use eyre::{Context, Result};
use serde::Deserialize;
use shared::{
	abis::{ExactInputSingleParams, ExactOutputSingleParams, Quoter, SwapRouter},
	coin::Coin,
};

use crate::{
	blockchain::{
		account::{Account, Unlocked},
		TradeProvider, Transaction, TransactionInfo,
	},
	TradeSignal,
};

const QUOTER_ADDRESS: &str = "0x61fFE014bA17989E743c5F6cB21bF9697530B21e";
const SWAP_ROUTER_ADDRESS: &str = "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45";

pub struct UniswapProvider {
	provider: Arc<Provider<Http>>,
	quoter: Quoter<Provider<Http>>,
	swap_router: SwapRouter<Provider<Http>>,
}

impl UniswapProvider {
	pub async fn swap(
		&self,
		transaction: &UniswapTransaction,
		account: &Account<Unlocked>,
	) -> Result<TransactionReceipt> {
		let TransactionInfo {
			amount_input,
			token_base,
			token_other,
			trade_type,
		} = &transaction.info;

		let contract_call = match trade_type {
			TradeSignal::Buy => {
				let token_in = token_base;
				let token_out = token_other;

				let params = ExactInputSingleParams {
					token_in: token_in.address,
					token_out: token_out.address,
					fee: transaction.fee,
					recipient: account.address,
					deadline: transaction.deadline.into(),
					amount_in: parse_units(amount_input, token_in.decimals)?.into(),
					amount_out_minimum: parse_units(transaction.minimum_amount_return, token_out.decimals)?
						.into(),
					sqrt_price_limit_x96: transaction.sqrt_price_limit,
				};

				Ok(self.swap_router.exact_input_single(params))
			}
			TradeSignal::Sell => {
				let token_in = token_other;
				let token_out = token_base;

				let params = ExactOutputSingleParams {
					token_in: token_in.address,
					token_out: token_out.address,
					fee: transaction.fee,
					recipient: account.address,
					deadline: transaction.deadline.into(),
					amount_out: parse_units(amount_input, token_out.decimals)?.into(),
					amount_in_maximum: parse_units(transaction.minimum_amount_return, token_in.decimals)?
						.into(),
					sqrt_price_limit_x96: transaction.sqrt_price_limit,
				};

				Ok(self.swap_router.exact_output_single(params))
			}
			TradeSignal::NoAction => Err(eyre!("swap was called with no action trade signal")),
		};

		let receipt: Option<TransactionReceipt> = contract_call?
			.send()
			.await?
			.inspect(|tx| println!("created transaction {:#?} for {:#x}", tx, account.address))
			.await?;

		let receipt = receipt.wrap_err("transaction could not be created")?;

		Ok(receipt)
	}
}

#[async_trait]
impl TradeProvider for UniswapProvider {
	async fn price(&self, base_coin: &Coin, coin: &Coin) -> Result<u32> {
		Ok(
			self
				.quoter
				.quote_exact_input_single(
					coin.address,
					base_coin.address,
					500,
					parse_units(1.0, coin.decimals)?.into(),
					U256::zero(),
				)
				.call()
				.await?
				.as_u32(),
		)
	}

	async fn combined_balance(&self, base_coin: &Coin, address: &Address) -> Result<u32> {
		todo!();
	}

	fn new() -> Result<Self> {
		let infura_secret = env::var("INFURA_SECRET").wrap_err("INFURA_SECRET should be in .env")?;
		let transport_url = format!("https://mainnet.infura.io/v3/{infura_secret}");
		let provider = Arc::new(Provider::<Http>::try_from(transport_url)?);

		Ok(Self {
			quoter: Quoter::new(Address::from_str(QUOTER_ADDRESS)?, provider.clone()),
			swap_router: SwapRouter::new(Address::from_str(SWAP_ROUTER_ADDRESS)?, provider.clone()),
			provider,
		})
	}
}

#[derive(Clone, Debug, Deserialize)]

pub struct UniswapTransaction {
	pub info: TransactionInfo,
	pub fee: u32,
	pub sqrt_price_limit: U256,
	pub minimum_amount_return: u32,
	pub deadline: i64,
}

impl Transaction for UniswapTransaction {
	fn verify(&self, _tx: TypedTransaction) -> Result<()> {
		Ok(()) // @TODO implement verification
	}

	fn new(info: TransactionInfo) -> Self {
		UniswapTransaction {
			info,
			fee: 500,
			sqrt_price_limit: U256::zero(), // @TODO set this value asap
			minimum_amount_return: 0,       // @TODO set this value asap
			deadline: (Utc::now() + Duration::minutes(10)).timestamp(),
		}
	}
}
