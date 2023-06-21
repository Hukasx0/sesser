use tokio::sync::{Mutex};
use std::net::SocketAddr;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::Service;
use hyper::{Request, Response, Method, body::Incoming};
use tokio::net::TcpListener;
use tokio::time::{interval, Duration};
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use http_body_util::BodyExt;
use serde::Deserialize;

mod database;
use database::Database;

#[derive(Deserialize)]
struct CreateTable {
    table_name: String,
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
                            if db.lock().await.check_table_exists(&form_data.table_name) {
                        //        println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Table name must be unique!"));
                            } else {
                                db.lock().await.create_table(&form_data.table_name);
                        //        println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Created table successfully!"));
                            }
                        }
                        Err(_) => {
                            return mk_response(format!("query /create accepts only unique database name"));
                        }
                    }
                },
                (&Method::POST, "/generate") => {
                    match serde_urlencoded::from_str::<GenerateValue>(&body_str.to_owned()) {
                        Ok(form_data) => {
                            if db.lock().await.check_table_exists(&form_data.table_name) {
                                let key = db.lock().await.generate_value(&form_data.table_name, form_data.expiration);
                          //      println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("{}", key));
                            } else {
                           //     println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Cannot add value to this table, because this table does not exist"));
                            }
                        }
                        Err(_) => {
                            return mk_response(format!("query /generate accepts only the name of the table in which the value is to be kept and the length of the interval after which the value is to expire"));
                        }
                    }
                }, 
                (&Method::POST, "/check") => {
                    match serde_urlencoded::from_str::<CheckValue>(&body_str.to_owned()) {
                        Ok(form_data) => {
                            if db.lock().await.check_table_exists(&form_data.table_name) {
                          //      println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("{}", db.lock().await.check_value_exists(&form_data.table_name, &form_data.value)));
                            } else {
                         //       println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Cannot check value in this table, because this table does not exist"));
                            }
                        }
                        Err(_) => {
                            return mk_response(format!("query /check only accepts the name of an existing table and the value stored in the table"));
                        }
                    }
                },
                (&Method::POST, "/check_table") => {
                    match serde_urlencoded::from_str::<CheckTable>(&body_str.to_owned()) {
                        Ok(form_data) => {
                            if db.lock().await.check_table_exists(&form_data.table_name) {
                                return mk_response("True".into());
                            } else {
                                return mk_response("False".into());
                            }
                        }
                        Err(_) => {
                            return mk_response(format!("query /check_table only accepts the name of an existing table"));
                        }
                    }
                },
                (&Method::POST, "/remove") => {
                    match serde_urlencoded::from_str::<RemoveValue>(&body_str.to_owned()) {
                        Ok(form_data) => {
                            if db.lock().await.check_table_exists(&form_data.table_name) {
                                db.lock().await.remove_value(&form_data.table_name, &form_data.value);
                          //      println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Removed value successfully!"));
                            } else {
                         //       println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Cannot remove value from a table that does not exist"));
                            }
                        }
                        Err(_) => {
                            return mk_response(format!("query /remove only accepts the existing table name and the value stored in the table"));
                        }
                    }
                },
                (&Method::POST, "/drop") => {
                    match serde_urlencoded::from_str::<DropTable>(&body_str.to_owned()) {
                        Ok(form_data) => {
                            if db.lock().await.check_table_exists(&form_data.table_name) {
                                db.lock().await.drop_table(&form_data.table_name);
                         //       println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("Removed table successfully!"));
                            } else {
                       //         println!("{}", format!("{:?}", db.lock().await)); // debug
                                return mk_response(format!("table with given name does not exist"));
                            }
                        }
                        Err(_) => {
                            return mk_response(format!("query /drop only accepts the name of an existing table"));
                        }
                    }
                },
                _ =>  mk_response("Unknown operation, available (only POST requests):\n/create\n/generate\n/check\n/check_table\n/remove\n/drop".into()),
            };
        x })
    }
}

type AsyncDatabase = Arc<Mutex<Database>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5379));
    let listener = TcpListener::bind(addr).await?;
    let database = Arc::new(Mutex::new(Database::new()));
    println!("Sesser works on http://127.0.0.1:5379");
    let db_clone = Arc::clone(&database); 
    tokio::spawn(async move {
        let mut filter_interval = interval(Duration::from_secs(120)); 
        loop {
            filter_interval.tick().await;
            db_clone.lock().await.filter_expired();
        }
    });
    loop {
        let (stream, _) = listener.accept().await?;
        let db_clone = Arc::clone(&database);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder {
                    db: db_clone,
                },)
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
