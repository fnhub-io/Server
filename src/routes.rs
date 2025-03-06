use crate::actors::{ExecuteFn, WasmEngineActor};
use crate::wasmFunction::run_wasm_function;
use actix::Addr;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::path::Path;
use actix_multipart::Multipart;
use futures::{StreamExt, TryStreamExt};

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
    dbg!(path);
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

#[post("/upload")]
async fn upload_fn(mut payload: Multipart) -> impl Responder {
    // Process multipart form data
    let mut fn_name = String::new();
    let mut wasm_data = Vec::new();
    
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition.as_ref()
            .and_then(|cd| cd.get_name())
            .unwrap_or("");
        
        if field_name == "fn_name" {
            // Process function name field
            while let Some(chunk) = field.next().await {
                if let Ok(data) = chunk.map(|b| b.to_vec()) {
                    if let Ok(s) = std::str::from_utf8(&data) {
                        fn_name = s.to_string();
                    }
                }
            }
        } else if field_name == "wasm_file" {
            // Process file field
            while let Some(chunk) = field.next().await {
                if let Ok(data) = chunk {
                    wasm_data.extend_from_slice(&data);
                }
            }
        }
    }
    
    if fn_name.is_empty() || wasm_data.is_empty() {
        return HttpResponse::BadRequest().body("Missing function name or empty WASM file");
    }
    
    // Save the file
    let path = Path::new("./src/savedWasmFunctions").join(&fn_name);
    
    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create directory: {}", e));
        }
    }
    
    // Write the file
    match std::fs::write(&path, &wasm_data) {
        Ok(_) => HttpResponse::Ok()
            .body(format!("Function '{}' uploaded successfully", fn_name)),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to write file: {}", e)),
    }
}
