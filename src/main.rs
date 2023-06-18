use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::SystemTime;
use tokio::sync::{Mutex};
use std::net::SocketAddr;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response, Method, body::Incoming};
use tokio::net::TcpListener;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;


fn sha2_hash(data: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.finalize().into()
}

#[derive(Debug)]
struct Responder {
    db: AsyncDatabase,
}

impl Service<Request<Incoming>> for Responder {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&mut self, req: Request<Incoming>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Full<Bytes>>, hyper::Error> {
            Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap())
        }

        let res = match (req.method(), req.uri().path()) {
            (&Method::POST, "/create") => mk_response(format!("Here you can create new hashmap of values\n{:?}", self)),
            (&Method::POST, "/generate") => mk_response(format!("Here you can generate new hash value by providing hashmap name and time limit\n{:?}", self)),
            (&Method::POST, "/remove") => mk_response(format!("Here you can remove a data from hashmap by providing a value\n{:?}", self)),
            (&Method::POST, "/drop") => mk_response(format!("Here yoy can remove hashmap by providing its name\n{:?}", self)),
            _ =>  return Box::pin(async { mk_response("Unknown operation, available (only POST requests):\n/create\n/generate\n/remove\n/drop".into()) }),
        };
        Box::pin(async { res })
    }
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

type AsyncDatabase = Arc<Mutex<Database>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5379));
    let listener = TcpListener::bind(addr).await?;
    println!("Sesser works on http://127.0.0.1:5379");
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder {
                    db: Arc::new(Mutex::new(HashMap::new())),
                },)
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
