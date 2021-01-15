#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
enum Health {
    Bar,
}
impl Health {}
use async_std::task;
use futures::channel::oneshot::{self, Sender};
use jsonrpsee_http_client::{HttpClient, HttpConfig};
use jsonrpsee_http_server::HttpServer;
use jsonrpsee_types::jsonrpc::{JsonValue, Params};
const SOCK_ADDR: &str = "127.0.0.1:9933";
const SERVER_URI: &str = "http://localhost:9933";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            {
                env_logger::init();
                let (server_started_tx, server_started_rx) = oneshot::channel::<()>();
                let _server = task::spawn(async move {
                    run_server(server_started_tx, SOCK_ADDR).await;
                });
                server_started_rx.await?;
                let client = HttpClient::new(SERVER_URI, HttpConfig::default())?;
                let response: Result<JsonValue, _> =
                    client.request("say_hello", Params::None).await;
                {
                    ::std::io::_print(::core::fmt::Arguments::new_v1(
                        &["r: ", "\n"],
                        &match (&response,) {
                            (arg0,) => {
                                [::core::fmt::ArgumentV1::new(arg0, ::core::fmt::Debug::fmt)]
                            }
                        },
                    ));
                };
                Ok(())
            }
        })
}
async fn run_server(server_started_tx: Sender<()>, url: &str) {
    let server = HttpServer::new(url, HttpConfig::default()).await.unwrap();
    let mut say_hello = server.register_method("say_hello".to_string()).unwrap();
    server_started_tx.send(()).unwrap();
    loop {
        let r = say_hello.next().await;
        r.respond(Ok(JsonValue::String("lo".to_owned())))
            .await
            .unwrap();
    }
}
