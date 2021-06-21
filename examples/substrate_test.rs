// Copyright 2019 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use jsonrpsee::{
	types::{to_json_value, JsonValue},
	ws_client::{traits::SubscriptionClient, v2::params::JsonRpcParams, Subscription, WsClient, WsClientBuilder},
};
use sp_storage::{well_known_keys, StorageKey};

const XT: &str = "0xe923f1ba03d2f40ea30743f7fbee384652c5e8459e64e06bf6248c5608efaa83";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	env_logger::init();
	let client = WsClientBuilder::default().build("ws://127.0.0.1:9944").await?;

	let keys = vec![StorageKey(well_known_keys::EXTRINSIC_INDEX.to_vec())];

	start_sub(&client, "chain_subscribeNewHead", JsonRpcParams::NoParams, "chain_unsubscribeNewHead", 3).await;
	start_sub(&client, "chain_subscribeAllHeads", JsonRpcParams::NoParams, "chain_unsubscribeAllHeads", 3).await;
	start_sub(&client, "chain_subscribeFinalizedHeads", JsonRpcParams::NoParams, "chain_unsubscribeFinalizedHeads", 3)
		.await;
	start_sub(
		&client,
		"state_subscribeStorage",
		vec![to_json_value(keys).unwrap()].into(),
		"state_unsubscribeStorage",
		3,
	)
	.await;
	// NOTE this will likely only get one notif
	// unless the version get updated.
	start_sub(&client, "state_subscribeRuntimeVersion", JsonRpcParams::NoParams, "state_unsubscribeRuntimeVersion", 1)
		.await;
	start_sub(
		&client,
		"author_submitAndWatchExtrinsic",
		vec![to_json_value(XT).unwrap()].into(),
		"author_unwatchExtrinsic",
		3,
	)
	.await;

	Ok(())
}

async fn start_sub(
	client: &WsClient,
	sub: &'static str,
	params: JsonRpcParams<'static>,
	unsub: &'static str,
	n: usize,
) {
	let mut s: Subscription<JsonValue> = client.subscribe(sub, params, unsub).await.unwrap();

	let mut i = 0;
	while i < n {
		let r = s.next().await;
		log::debug!("received {:?}", r);
		i += 1;
	}

	drop(s);
	std::thread::sleep(std::time::Duration::from_secs(1));
}
