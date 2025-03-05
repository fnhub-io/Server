mod wasmFunction;
mod actors;
mod routes;
mod minio;

use actors::WasmEngineActor;
use actix_web::{App, HttpServer, web};
use actix::prelude::*;
use routes::{execute_fn, test, retrieve_wasm_file};
use crate::minio::create_minio_client;
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let minio_client = match create_minio_client().await {
        Ok(client) => Arc::new(Mutex::new(Some(client))),
        Err(e) => {
            eprintln!("Failed to create MinIO client: {}", e);
            Arc::new(Mutex::new(None))
        }
    };

    
    let wasm_actor = WasmEngineActor{}.start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(wasm_actor.clone()))
            .service(test)
            .service(execute_fn)
            .service(retrieve_wasm_file)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
