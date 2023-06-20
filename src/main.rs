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
use http_body_util::BodyExt;
use serde::Deserialize;

mod database;
use database::Database;

#[derive(Deserialize)]
struct CreateTable {
    name: String,
}

#[derive(Deserialize)]
struct GenerateValue {
    table_name: String,
    expiration: u64,
}

#[derive(Deserialize)]
struct CheckValue {
    table_name: String,
    value: String,
}

#[derive(Deserialize)]
struct CheckTable {
    table_name: String,
}

#[derive(Deserialize)]
struct RemoveValue {
    table_name: String,
    value: String,
}

#[derive(Deserialize)]
struct DropTable {
    table_name: String,
}

#[derive(Debug, Clone)]
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

        let self_clone = self.clone();
        let db = Arc::clone(&self_clone.db);

        Box::pin(async move { 
            let method = req.method().clone();
            let path = req.uri().path().to_owned();
            let body_bytes = &req.collect().await?.to_bytes().iter().cloned().collect::<Vec<u8>>();
            let body_str = String::from_utf8_lossy(&body_bytes).to_string();
            let x = match (&method, path.as_str()) {
                (&Method::POST, "/create") =>  {
                    match serde_urlencoded::from_str::<CreateTable>(&body_str.to_owned()) {
                        Ok(form_data) => {
                            if db.lock().await.check_table_exists(&form_data.name) {
                                println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Table name must be unique!\n"));
                            } else {
                                db.lock().await.create_table(&form_data.name);
                                println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Created table successfully!\n"));
                            }
                        }
                        Err(_) => {
                            return mk_response(format!("query /create accepts only unique database name\n"));
                        }
                    }
                },
                (&Method::POST, "/generate") => mk_response(format!("Here you can generate new hash value by providing hashmap name and time limit\n")),
                (&Method::POST, "/check") => mk_response(format!("Here you can check if value exists in HashMap, and get True or False\n")),
                (&Method::POST, "/check_table") => mk_response(format!("here you can check hashmap with given name exists, and get True or False\n")),
                (&Method::POST, "/remove") => mk_response(format!("Here you can remove a data from hashmap by providing a value\n")),
                (&Method::POST, "/drop") => mk_response(format!("Here yoy can remove hashmap by providing its name\n")),
                _ =>  mk_response("Unknown operation, available (only POST requests):\n/create\n/generate\n/remove\n/drop".into()),
            };
        x })
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
//type Database = HashMap<String,HashMap<String,Option<SystemTime>>>;

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
                    db: Arc::new(Mutex::new(Database::new())),
                },)
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
