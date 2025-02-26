mod wasmFunction;
mod actors;
mod routes;

use actors::{ExecuteFn, WasmEngineActor};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use actix::prelude::*;
use routes::{execute_fn, test};



#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let wasm_actor = WasmEngineActor{}.start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(wasm_actor.clone()))
            .service(test)
            .service(execute_fn)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}