use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::Mutex;
use std::convert::Infallible;
use std::net::SocketAddr;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use tokio::net::TcpListener;


fn sha2_hash(data: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.finalize().into()
}

async fn server_response(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello world!"))))
}

/* Database :=
    hashmap< 
            name - for example 'sessions' or 'api_keys',
             hashmap< for holding multiple values, for example multiple api keys
                value
                expiration date
             >>
*/
type Database = HashMap<String,HashMap<String,Option<SystemTime>>>;

type AsyncDatabase = Mutex<Database>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5379));
    let listener = TcpListener::bind(addr).await?;
    println!("Sesser works on http://127.0.0.1:5379");
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, service_fn(server_response))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
