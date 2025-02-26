use crate::actors::{WasmEngineActor, ExecuteFn};
use crate::wasmFunction::run_wasm_function;
use actix::Addr;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::path::Path;

#[get("/")]
async fn test() -> impl Responder {
    let port = std::env::var("PORT").unwrap_or("8080".to_string()).parse::<u16>().unwrap();
    HttpResponse::Ok().body(format!("The server is alive and running at port {}!", port))
}

#[post("/execute")]
async fn execute_fn(
    fn_name: String,
    wasm_actor: web::Data<Addr<WasmEngineActor>>,
    ) -> impl Responder {
        let addr = format!("/home/arjun/Desktop/mini-project/backend/server/src/savedWasmFunctions/{}", fn_name);
        let path = Path::new(&addr);
        if path.exists() {
            let output = wasm_actor.send(ExecuteFn { name: fn_name }).await.unwrap();
            match output {
                Ok(content) => HttpResponse::Ok().body(content),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        } else {
            HttpResponse::NotFound().body("Function not found")
        }
}