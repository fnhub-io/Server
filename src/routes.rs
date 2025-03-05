use crate::actors::{ExecuteFn, WasmEngineActor};
use crate::wasmFunction::run_wasm_function;
use crate::minio::{MinioClient, create_minio_client};
use actix::Addr;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;


lazy_static::lazy_static! {
    static ref MINIO_CLIENT: Arc<Mutex<Option<MinioClient>>> = Arc::new(Mutex::new(None));
}

#[get("/")]
async fn test() -> impl Responder {
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse::<u16>()
        .unwrap();
    HttpResponse::Ok().body(format!("The server is alive and running at port {}!", port))
}

#[post("/execute")]
async fn execute_fn(
    fn_name: String,
    wasm_actor: web::Data<Addr<WasmEngineActor>>,
) -> impl Responder {
    let addr = format!("./src/savedWasmFunctions/{}", fn_name);
    let path = Path::new(&addr);
    
    if path.exists() {
        
        let output = wasm_actor.send(ExecuteFn { name: fn_name.clone() }).await.unwrap();
        

        let mut minio_client = MINIO_CLIENT.lock().await;
        if let Some(client) = minio_client.as_ref() {
            match client.store_wasm_file(&addr, &fn_name).await {
                Ok(_) => println!("Successfully stored WASM file in MinIO"),
                Err(e) => eprintln!("Error storing WASM file in MinIO: {}", e),
            }
        }
        
        match output {
            Ok(content) => HttpResponse::Ok().body(content),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    } else {
        HttpResponse::NotFound().body("Function not found")
    }
}

#[post("/retrieve-wasm")]
async fn retrieve_wasm_file(
    web::Json(file_name): web::Json<String>
) -> impl Responder {
    let mut minio_client = MINIO_CLIENT.lock().await;
    if let Some(client) = minio_client.as_ref() {
        let local_path = format!("./src/savedWasmFunctions/{}", file_name);
        match client.retrieve_wasm_file(&file_name, &local_path).await {
            Ok(_) => HttpResponse::Ok().body("WASM file retrieved successfully"),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    } else {
        HttpResponse::InternalServerError().body("MinIO client not initialized")
    }
}
