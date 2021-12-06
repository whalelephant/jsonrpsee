// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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
	rpc_params,
	types::{traits::SubscriptionClient, Subscription, JsonValue},
	ws_client::WsClientBuilder,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::FmtSubscriber::builder()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()
		.expect("setting default subscriber failed");

	let url = "ws://127.0.0.1:9944";
	for _ in 0..1 {
		// default max connections is 100.
		run_concurrent_subscriptions(100, url, true).await;
		run_concurrent_subscriptions(100, url, false).await;
	}

	Ok(())
}

async fn run_concurrent_subscriptions(n: usize, url: &'static str, graceful_shutdown: bool) {
	let mut tasks = Vec::new();
	for _ in 0..n {
		let url = url.clone();
		tasks.push(tokio::spawn(async move {
			let client = WsClientBuilder::default().build(&url).await.unwrap();
			let mut sub: Subscription<JsonValue> = client.subscribe("state_subscribeStorage", rpc_params![], "state_unsubscribeStorage").await.unwrap();
			
			// Make sure we the `initial response` and once more.
			let r1 = sub.next().await.unwrap();
			let r2 = sub.next().await.unwrap();
			
			// This will not call the `unsubscribe call`.
			if !graceful_shutdown {
				drop(client);
			}

			tracing::debug!("{:?}", r1);
			tracing::debug!("{:?}", r2);
		}));
	}
	futures::future::join_all(tasks).await;
}

