
use serde_json::Value;
use std::net::SocketAddr;
use hyper::Client;
use hyper::rt::Future;
use hyper_tls::HttpsConnector;

use futures::future;
use hyper::{self, Body, Request, Response, StatusCode};


type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(Clone)]
pub struct Service {
    conf: Value,
    pub addr: SocketAddr
}

impl Service {
    pub fn new(conf: Value) -> Self {
        let addr = conf["listen_on"].as_str().unwrap().parse().expect("Failed to parse listening address");
        Service {
            conf: conf.clone(),
            addr
        }
    }

    pub fn handle(&self, req: Request<Body>) -> BoxFut {
        let mut response = Response::new(Body::empty());
        let path = req.uri().path(); 
        println!("Incoming request {:?}, {}", req.method(), path);
        if !path.starts_with("/http") { *response.status_mut() = StatusCode::NOT_FOUND; }
        else {
            match path[1..].parse() {
                Ok(url) => {
                    println!("Fetching {}", url);
                    let fetching = match path.starts_with("/https") {
                        true => {
                            let https = HttpsConnector::new(4).expect("TLS initialization failed");
                            Client::builder().build::<_, hyper::Body>(https).get(url)
                        },
                        _ => Client::new().get(url)
                    };
                    let fetch = fetching
                        .and_then(|res| {
                            // println!("Response: {}", res.status());
                            // println!("Headers: {:#?}", res.headers());
                            *response.body_mut() = res.into_body();
                            Ok(response)
                        });
                        return Box::new(fetch);
                },
                Err(_) => {
                    println!("Invalid url: {}", path);
                    *response.status_mut() = StatusCode::NOT_FOUND;
                }
            }

        }
        Box::new(future::ok(response))
    }
}




