use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::Mutex;
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

fn sha2_hash(data: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.finalize().into()
}

async fn server_response(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello world!".into()))
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
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5379));
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(server_response))
    });
    let server = Server::bind(&addr).serve(make_svc);
    println!("Sesser works on http://127.0.0.1:5379");
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
