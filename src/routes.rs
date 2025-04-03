use crate::actors::{ExecuteFn, WasmEngineActor};
use crate::wasmFunction::FunctionMetrics;
use actix::Addr;
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};

use minio::s3::{
    args::{DownloadObjectArgs, PutObjectArgs},
    client::Client,
};
use tokio::fs::remove_file;

use std::{fs::File, path::Path};
use std::path::Path;
use std::fs;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
struct ExecutePayload {
    fn_name: String,
    params: Vec<String>,
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
    web::Json(payload): web::Json<ExecutePayload>,
    wasm_actor: web::Data<Addr<WasmEngineActor>>,
    client: web::Data<Client>,
) -> impl Responder {
    let bucket_name = "neelabucket";
    let object_name = &payload.fn_name;
    let download_path = format!("./src/cache/{}", payload.fn_name);
    let path = Path::new(&download_path);

    // Download the file from MinIO

    // let args = GetObjectArgs::new(bucket_name, object_name).unwrap();
    // let get_object = client.get_object(&args).await;
    let args = DownloadObjectArgs::new(bucket_name, object_name, download_path.as_str()).unwrap();
    client.download_object(&args).await.unwrap();
    // match get_object {
    //     Ok(object) => {
    //         object.content.to_file(path).await.unwrap();
    //     }
    //     Err(e) => {
    //         return HttpResponse::InternalServerError()
    //             .body(format!("Failed to download file: {}", e));
    //     }
    // }

    if path.exists() {
        let output = wasm_actor
            .send(ExecuteFn {
                name: payload.fn_name.clone(),
                params: payload.params,
            })
            .await
            .unwrap();
        remove_file(path).await.unwrap();
        match output {
            Ok(content) => HttpResponse::Ok().body(content),
            Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
        }
    } else {
        HttpResponse::NotFound().body("Function not found")
    }
}

#[post("/upload")]
async fn upload_fn(mut payload: Multipart, client: web::Data<Client>) -> impl Responder {
    // Process multipart form data
    dbg!("Upload being called");
    let mut fn_name = String::new();
    let mut wasm_data = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let field_name = content_disposition
            .as_ref()
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

    // Save the file temporarily
    let path = Path::new("./src/cache").join(&fn_name);

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create directory: {}", e));
        }
    }

    // Write the file
    match std::fs::write(&path, &wasm_data) {
        Ok(_) => {
            // Upload the file to MinIO
            let bucket_name = "neelabucket";
            let object_name = &fn_name;
            // let content = ObjectContent::from(path.clone());

            let meta = std::fs::metadata(&path).unwrap();
            let object_size = Some(meta.len() as usize);
            let mut file = File::open(&path).unwrap();
            let mut args =
                PutObjectArgs::new(bucket_name, object_name, &mut file, object_size, None).unwrap();

            match client.put_object(&mut args).await {
                Ok(_) => {
                    // Delete the temporary file
                    std::fs::remove_file(&path).unwrap();
                    HttpResponse::Ok().body(format!("Function '{}' uploaded successfully", fn_name))
                }
                Err(e) => HttpResponse::InternalServerError()
                    .body(format!("Failed to upload file to MinIO: {}", e)),
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Failed to write file: {}", e)),
    }
}


#[get("/metrics")]
async fn get_metrics() -> impl Responder {
    let metrics_path = "function_metrics.json";
    match fs::read_to_string(metrics_path) {
        Ok(content) => {
            let metrics: HashMap<String, FunctionMetrics> = 
                serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new());
            HttpResponse::Ok().json(metrics)
        },
        Err(_) => {
            // Return empty metrics if file doesn't exist yet
            HttpResponse::Ok().json(HashMap::<String, FunctionMetrics>::new())
        }
    }
}

#[get("/metrics/{fn_name}")]
async fn get_function_metrics(path: web::Path<String>) -> impl Responder {
    let fn_name = path.into_inner();
    let metrics_path = "function_metrics.json";
    match fs::read_to_string(metrics_path) {
        Ok(content) => {
            let metrics: HashMap<String, FunctionMetrics> = 
                serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new());
            
            if let Some(function_metrics) = metrics.get(&fn_name) {
                HttpResponse::Ok().json(function_metrics)
            } else {
                HttpResponse::NotFound().body(format!("No metrics found for function: {}", fn_name))
            }
        },
        Err(_) => {
            HttpResponse::NotFound().body("Metrics data not found")
        }
    }
}
