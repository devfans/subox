
extern crate serde_json;
extern crate hyper;
extern crate futures;
extern crate hyper_tls;

use serde_json::Value;
use hyper::Server;
use hyper::service::service_fn;

mod service;
use service::*;
use std::fs;
use std::env;
use futures::Future;
use std::sync::Arc;

fn main() {
    let mut conf_path = None;

    for arg in env::args().skip(1) {
        if arg.starts_with("-c") {
            conf_path = Some(arg.split_at(14).1.to_string());
        }
    }

    let conf = if let Some(path) = conf_path {
        path
    } else {
        "./config.json".to_string()
    };
    println!("Loading configuration from {}", conf);

    let conf_file = fs::File::open(conf).expect("Failed to read config file");
    let config: Value= serde_json::from_reader(conf_file).expect("Failed to parse config file");
    let service = Service::new(config);
    let service_ref = Arc::new(service.clone());
    let listen_addr = service.addr;
    let server = Server::bind(&service.addr).serve(move || {
        let service_ref = service_ref.clone();
        service_fn(move |req| service_ref.handle(req))
    }).map_err(|e| println!("server error: {}", e));
    println!("Server is listening on {}", listen_addr);
    hyper::rt::run(server);
}
