use std::marker::PhantomData;

use ethers::types::Address;
use eyre::Result;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use shared::coin::Coin;

#[derive(Clone, Debug, Deserialize)]
pub struct Locked;

#[derive(Clone, Debug, Deserialize)]
pub struct Unlocked;

#[derive(Deserialize, Clone, Debug)]
pub struct Account<State = Locked> {
	pub address: Address,
	pub algorithm: Option<ObjectId>,
	pub coin: Coin,
	private_key: String,
	state: PhantomData<State>,
}

impl Account<Locked> {
	pub fn unlock(&self) -> Result<Account<Unlocked>> {
		todo!("account decryption tbd")
	}
}

impl Account<Unlocked> {
	pub fn lock(&self) -> Result<Account<Locked>> {
		todo!("account decryption tbd")
	}

	pub fn private_key(&self) -> &str {
		&self.private_key
	}
}
