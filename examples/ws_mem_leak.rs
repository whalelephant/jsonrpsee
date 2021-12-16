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
use futures::stream::{FuturesUnordered, StreamExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing_subscriber::FmtSubscriber::builder()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
		.try_init()
		.expect("setting default subscriber failed");

	let url = "ws://127.0.0.1:9944";
	for _ in 0..1 {
		// default max connections is 100.
		run_concurrent_subscriptions(1, url, true).await;
		// run_concurrent_subscriptions(100, url, false).await;
	}

	tokio::time::sleep(std::time::Duration::from_secs(10)).await;

	Ok(())
}

async fn run_concurrent_subscriptions(n: usize, url: &'static str, graceful_shutdown: bool) {
    let keys: Vec<_> = (0..2)
        .map(|nth| {
            format!(
                "0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045d{:04x}",
                nth
            )
        })
        .collect();


    let mut futures: FuturesUnordered<_> = (0..n)
        .map(|_| WsClientBuilder::default().build(url))
        .collect();

	let mut clients = Vec::new();
    while let Some(client) = futures.next().await {
        match client {
            Ok(client) => clients.push(client),
            Err(error) => tracing::error!("Failed to open connection: {}", error)
        }
    }

    let mut futures_storage = FuturesUnordered::new();
    for client in &mut clients {
        for _ in 0..2 {
            futures_storage.push(client.subscribe::<JsonValue>(
                "state_subscribeStorage",
                rpc_params![&keys],
                "state_unsubscribeStorage",
            ));
        }
    }

	let mut subs = Vec::new();

	while let Some(subscription) = futures_storage.next().await {
        if let Ok(subscription) = subscription {
			subs.push(subscription);
        } else {
			panic!("err");
		}
    }

	tokio::time::sleep(std::time::Duration::from_secs(30)).await;

	std::mem::drop(subs);
}

