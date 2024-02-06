use std::{
	env,
	io::{BufRead, BufReader, Write},
	process::{ChildStdin, ChildStdout, Command, Stdio},
};

use ethers::{
	prelude::Http,
	providers::{Middleware, Provider},
	types::{Bytes, TransactionReceipt, TransactionRequest, U256},
	utils::hex::FromHex,
};
use eyre::{Context, OptionExt, Result};

use crate::blockchain::TransactionInfo;

const CHAIN_ID: u16 = 0x5;

pub struct UniswapPool(ChildStdin, Provider<Http>);
pub struct UniswapPoolEntry {
	calldata: Bytes,
	value: U256,
}

impl UniswapPool {
	async fn execute_transaction(&self, entry: UniswapPoolEntry) -> Result<TransactionReceipt> {
		let provider = &self.1;
		let tx = TransactionRequest::new()
			.data(entry.calldata)
			.value(entry.value);

		provider
			.send_transaction(tx, None)
			.await?
			.log_msg("pending transaction")
			.await?
			.ok_or_eyre("no transaction")
	}

	async fn process_lines(&self, stdout: ChildStdout) -> Result<()> {
		let reader = BufReader::new(stdout);

		let _ = reader
			.lines()
			.filter_map(|line| line.ok())
			.for_each(|line: String| {
				let split: Vec<&str> = line.split(':').collect();
				if let (Ok(calldata), Ok(value)) = (
					Bytes::from_hex(split[0]),
					U256::from_str_radix(split[1], 16),
				) {
					let entry = UniswapPoolEntry { calldata, value };

					let _ = self.execute_transaction(entry);
				}
			});

		Ok(())
	}

	pub fn push(&mut self, transaction: &TransactionInfo) -> Result<()> {
		Ok(
			self
				.0
				.write_all(serde_json::to_string(transaction)?.as_bytes())?,
		)
	}

	pub fn new() -> Result<Self> {
		let infura_secret = env::var("INFURA_SECRET").wrap_err("INFURA_SECRET should be in .env")?;
		let transport_url = format!("https://goerli.infura.io/v3/{infura_secret}");
		let transport_url = transport_url.as_str();

		let provider = Provider::<Http>::try_from(transport_url)?;

		let router = Command::new("node")
			.args([
				"transaction-router/",
				transport_url,
				CHAIN_ID.to_string().as_str(),
			])
			.stdout(Stdio::piped())
			.stdin(Stdio::piped())
			.spawn()?;

		let stdout = router.stdout.ok_or_eyre("no stdout")?;
		let stdin = router.stdin.ok_or_eyre("no stdin")?;

		let pool = Self(stdin, provider);
		let _ = pool.process_lines(stdout);

		Ok(pool)
	}
}
