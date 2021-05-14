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
	ws_client::{traits::{Client, SubscriptionClient}, v2::params::JsonRpcParams, Subscription, WsClientBuilder},
	ws_server::WsServer,
};
use std::net::SocketAddr;
use serde_json;

const NUM_SUBSCRIPTION_RESPONSES: usize = 10;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	env_logger::init();
	let addr = run_server().await?;
	let url = format!("ws://{}", addr);

	let client = WsClientBuilder::default().build(&url).await?;
	let mut subscribe_hello: Subscription<String> =
		client.subscribe("subscribe_hello", JsonRpcParams::NoParams, "unsubscribe_hello").await?;

	let mut i = 0;
	while i <= NUM_SUBSCRIPTION_RESPONSES {
		let r = subscribe_hello.next().await;
		log::debug!("received {:?}", r);
		i += 1;
	}

	let r = subscribe_hello.next().await;
	println!("Got one more ({:?})", r);
	// This is painful
	// let o = client.request::<String>("unsubscribe_hello", JsonRpcParams::Array(vec![subscribe_hello.id.clone().into()])).await;
	// Can't `SubscriptionId` be `Copy`?
	let sub_id: serde_json::Value = subscribe_hello.id.clone().into();
	let o = client.request::<String>("unsubscribe_hello", vec![sub_id].into()).await;
	println!("Unsubscribed now? {:?}", o);
	// This hangs
	let r = subscribe_hello.next().await;
	println!("No more ({:?})", r);

	Ok(())
}

async fn run_server() -> anyhow::Result<SocketAddr> {
	let mut server = WsServer::new("127.0.0.1:0").await?;
	let mut subscription = server.register_subscription("subscribe_hello", "unsubscribe_hello").unwrap();

	std::thread::spawn(move || loop {
		subscription.send(&"hello my friend").unwrap();
		std::thread::sleep(std::time::Duration::from_secs(1));
	});

	let addr = server.local_addr();
	tokio::spawn(async move { server.start().await });
	addr
}
