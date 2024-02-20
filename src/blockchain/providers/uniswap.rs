use std::{
	env,
	process::{Child, Command, Stdio},
	time::Duration,
};

use async_trait::async_trait;
use ethers::{
	middleware::MiddlewareBuilder,
	prelude::Http,
	providers::{Middleware, Provider},
	signers::{LocalWallet, Signer},
	types::{Address, Bytes, TransactionReceipt, TransactionRequest, U256},
	utils::hex::FromHex,
};
use eyre::{eyre, Context, OptionExt, Result};
use reqwest::StatusCode;
use shared::{abis::ERC20, coin::Coin};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, instrument, trace};

use crate::{
	blockchain::{
		account::{Account, Locked},
		chain_id, TradeProvider, TransactionInfo,
	},
	TradeSignal,
};

struct ChildGuard(Child);

impl Drop for ChildGuard {
	fn drop(&mut self) {
		match self.0.kill() {
			Err(error) => error!(error = ?error, child = ?self.0, "could not kill child process"),
			Ok(_) => debug!(child = ?self.0, "killed child process"),
		}
	}
}

pub struct UniswapProvider(Provider<Http>, ChildGuard);

#[derive(Debug, Clone)]
pub struct UniswapPoolEntry {
	calldata: Bytes,
	value: U256,
}

async fn try_connection(
	connection_timeout: Duration,
	retry_timeout: Duration,
	url: &str,
) -> Result<()> {
	trace!("running connection attempt loop");
	loop {
		let timeout_result = timeout(connection_timeout, reqwest::get(url)).await;
		if let Ok(Ok(response)) = timeout_result {
			trace!(response = ?response, "response succeeded");
			return Ok(());
		}
		trace!("retrying connection");
		sleep(retry_timeout).await;
	}
}

// timeout controls the entire function, retry_timeout controls each tcp connection
#[instrument(level = "trace")]
async fn wait_for_connection(
	url: &str,
	timeouts: (Option<Duration>, Option<Duration>, Option<Duration>),
) -> Result<()> {
	let main_timeout = match timeouts.0 {
		Some(timeout) => timeout,
		None => Duration::from_secs(30),
	};
	let connection_timeout = match timeouts.1 {
		Some(timeout) => timeout,
		None => Duration::from_secs(5),
	};
	let retry_timeout = match timeouts.2 {
		Some(timeout) => timeout,
		None => Duration::from_millis(500),
	};

	match timeout(
		main_timeout,
		try_connection(connection_timeout, retry_timeout, url),
	)
	.await
	{
		Ok(_) => {
			trace!("connection resolved");
			Ok(())
		}
		Err(error) => {
			trace!("main timeout triggered");
			Err(error.into())
		}
	}
}

impl UniswapProvider {
	async fn approve(&self, coin: &Coin, wallet: LocalWallet) -> Result<TransactionReceipt> {
		let contract = ERC20::new(
			coin.address,
			self
				.0
				.clone()
				.with_signer(wallet.with_chain_id(chain_id()))
				.into(),
		);

		let receipt = contract
			.approve(
				env::var("ROUTER_ADDRESS")
					.expect("ROUTER_ADDRESS should be in .env")
					.parse::<Address>()?,
				U256::MAX,
			)
			.send()
			.await?
			.await?
			.ok_or_eyre("no transaction")?;

		Ok(receipt)
	}

	async fn need_approval(&self, coin: &Coin, account: &Account) -> Result<bool> {
		let contract = ERC20::new(coin.address, self.0.clone().into());

		let allowance: U256 = contract
			.allowance(
				account.address,
				env::var("ROUTER_ADDRESS")
					.expect("ROUTER_ADDRESS should be in .env")
					.parse::<Address>()?,
			)
			.call()
			.await?;

		Ok(allowance < U256::MAX / U256::from(1_000_000_000))
	}
}

#[async_trait]
impl TradeProvider for UniswapProvider {
	async fn verify(&self) -> Result<()> {
		// @TODO implement transaction verification
		// balance + gas checks, sqrtPriceLimit checks, max/minOutChecks, ect
		Ok(())
	}

	#[instrument(err, skip(self))]
	async fn transact(
		&mut self,
		transaction: &TransactionInfo,
		account: Account<Locked>,
	) -> Result<()> {
		debug!(transaction = ?transaction, "fetching tx route");
		let response = reqwest::Client::new()
			.post("http://localhost:6278/route")
			.json(transaction)
			.send()
			.await?;

		let status = response.status();

		debug!(response = ?response, "route response data");
		let route = response.text().await?;

		if status >= StatusCode::from_u16(400)? {
			Err(eyre!("status code {} recieved with data {}", status, route))?;
		}

		let split: Vec<&str> = route.split(':').collect();
		let calldata = Bytes::from_hex(split[0])?;
		let value = U256::from_str_radix(split[1], 16)?;

		let entry = UniswapPoolEntry { calldata, value };

		let provider = &self.0;
		let request = TransactionRequest::new()
			.data(entry.calldata)
			.value(entry.value)
			.to(
				env::var("ROUTER_ADDRESS")
					.expect("ROUTER_ADDRESS should be in .env")
					.parse::<Address>()?,
			)
			.from(account.address);

		debug!(request = ?request, "processing route data into TransactionRequest");

		let wallet = account.unlock()?.private_key().parse::<LocalWallet>()?;

		let approve_coin = {
			match transaction.action {
				TradeSignal::Buy => &transaction.pair.0,
				TradeSignal::Sell => &transaction.pair.1,
				TradeSignal::NoAction => panic!("no action when choosing approval coin"),
			}
		};

		if self.need_approval(approve_coin, &account).await? {
			let receipt = self.approve(approve_coin, wallet.clone()).await?;
			info!(receipt = ?receipt, "approval completed");
		}

		let receipt = provider
			.clone()
			.with_signer(wallet.with_chain_id(chain_id()))
			.send_transaction(request, None)
			.await?
			.await?
			.ok_or_eyre("no transaction")?;
		info!(receipt = ?receipt, "transaction completed");

		Ok(())
	}

	#[instrument(err, name = "new_provider", level = "debug")]
	async fn new() -> Result<Self> {
		let infura_secret = env::var("INFURA_SECRET").wrap_err("INFURA_SECRET should be in .env")?;
		let transport_url = format!("https://goerli.infura.io/v3/{infura_secret}");
		let transport_url = transport_url.as_str();

		let provider = Provider::<Http>::try_from(transport_url)?;

		let router = Command::new("node")
			.args([
				"transaction-router/",
				transport_url,
				chain_id().to_string().as_str(),
			])
			.stdout(Stdio::inherit())
			.stderr(Stdio::inherit())
			.stdin(Stdio::null())
			.spawn()?;
		debug!("spawned new transaction router");

		let router = ChildGuard(router);

		wait_for_connection("http://localhost:6278/ping", (None, None, None)).await?;

		Ok(Self(provider, router))
	}
}
