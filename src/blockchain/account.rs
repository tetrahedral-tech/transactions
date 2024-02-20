use std::{env, marker::PhantomData, str};

use ethers::types::Address;
use eyre::Result;
use faster_hex::hex_decode;
use mongodb::bson::oid::ObjectId;
use openssl::symm::{decrypt, Cipher};
use serde::Deserialize;
use tracing::instrument;

#[derive(Clone, Debug, Deserialize)]
pub struct Locked;

#[derive(Clone, Debug, Deserialize)]
pub struct Unlocked;

#[derive(Deserialize, Clone, Debug)]
pub struct Account<State = Locked> {
	pub address: Address,
	pub algorithm: ObjectId,
	pub interval: u16,
	pub pair: (String, String),
	#[serde(rename = "encryptedPrivateKey")]
	private_key: String,
	#[serde(skip)]
	state: PhantomData<State>,
}

impl Account<Locked> {
	#[instrument(err)]
	pub fn unlock(&self) -> Result<Account<Unlocked>> {
		let data: Vec<&str> = self.private_key.split(':').collect();
		let (key_hex, iv_hex, encrypted_hex) = (
			env::var("WALLET_SECRET").expect("WALLET_SECRET should be in .env"),
			data[0].as_bytes(),
			data[1].as_bytes(),
		);

		let key_hex = key_hex.as_bytes();

		let (mut key, mut iv, mut encrypted) = (
			vec![0; key_hex.len() / 2],
			vec![0; iv_hex.len() / 2],
			vec![0; encrypted_hex.len() / 2],
		);

		hex_decode(key_hex, &mut key)?;
		hex_decode(iv_hex, &mut iv)?;
		hex_decode(encrypted_hex, &mut encrypted)?;

		let private_key = decrypt(Cipher::aes_256_cbc(), &key, Some(&iv), &encrypted)?;

		Ok(Account {
			address: self.address,
			algorithm: self.algorithm,
			interval: self.interval,
			pair: self.pair.clone(),
			private_key: String::from_utf8(private_key)?,
			state: PhantomData::<Unlocked>,
		})
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
