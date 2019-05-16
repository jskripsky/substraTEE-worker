/*
   Copyright 2019 Supercomputing Systems AG

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

	   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

*/

extern crate substrate_api_client;
extern crate runtime_primitives;
extern crate my_node_runtime;
extern crate parity_codec;
extern crate primitives;
extern crate hex_literal;
extern crate sgx_crypto_helper;
extern crate env_logger;
extern crate log;

use parity_codec::{Encode};
use substrate_api_client::{Api};

use primitive_types::U256;
use blake2_rfc::blake2s::{blake2s};

use substratee_client::*;

#[macro_use]
extern crate clap;
use clap::App;

fn main() {
	env_logger::init();

	let yml = load_yaml!("cli.yml");

	let matches = App::from_yaml(yml).get_matches();
	if let Some(_matches) = matches.subcommand_matches("getcounter") {
		let user = "Alice";
		println!("*** Getting the counter value of '{}' from the substraTEE-worker", user);
		get_counter(user);
		return;
	}

	let port = matches.value_of("port").unwrap_or("9944");
	let server = matches.value_of("server").unwrap_or("127.0.0.1");
	let mut api = Api::new(format!("ws://{}:{}", server, port));
	api.init();

	// get Alice's free balance
	get_free_balance(&api, "//Alice");

	// get Alice's account nonce
	let mut nonce = get_account_nonce(&api, "//Alice");

	// fund the account of Alice
	fund_account(&api, "//Alice", 1_000_000, nonce, api.genesis_hash.unwrap());

	// transfer from Alice to TEE
	nonce = get_account_nonce(&api, "//Alice");
	let tee_pub = get_enclave_ecc_pub_key();
	transfer_amount(&api, "//Alice", tee_pub, U256::from(1000), nonce, api.genesis_hash.unwrap());

	// compose extrinsic with encrypted payload
	let rsa_pubkey = get_enclave_rsa_pub_key();
	let mut payload_encrypted: Vec<u8> = Vec::new();
	let message = matches.value_of("message").unwrap_or("Alice,42");
	let plaintext = message.as_bytes();
	rsa_pubkey.encrypt_buffer(&plaintext, &mut payload_encrypted).unwrap();
	println!("[>] Sending message {:?} to substraTEE-worker", message);
	nonce = get_account_nonce(&api, "//Alice");
	let xt = compose_extrinsic_substratee_call_worker("//Alice", payload_encrypted, nonce, api.genesis_hash.unwrap());
	let mut _xthex = hex::encode(xt.encode());
	_xthex.insert_str(0, "0x");

	// send and watch extrinsic until finalized
	let tx_hash = api.send_extrinsic(_xthex).unwrap();
	println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
	println!("[<] Message sent successfully");
	println!("");

	// subsribe to callConfirmed event
	println!("[>] Subscribe to callConfirmed event");
	let act_hash = subscribe_to_call_confirmed(api);
	println!("[<] callConfirmed event received");
	println!("[+] Expected Hash: {:?}", blake2s(32, &[0; 32], &plaintext).as_bytes());
	println!("[+] Actual Hash:   {:?}", act_hash);
}
