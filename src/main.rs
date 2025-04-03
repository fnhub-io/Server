mod actors;
mod routes;
mod wasm_function;

use actix::prelude::*;
use actix_cors::Cors;
use actix_web::{get, post, HttpResponse, Responder};
use actix_web::{middleware, web, App, HttpServer};
use actors::WasmEngineActor;
use minio::s3::args::{BucketExistsArgs, MakeBucketArgs};
use minio::s3::client::ClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use routes::{execute_fn, test, upload_fn};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start WasmEngineActor
    let wasm_actor = WasmEngineActor {}.start();

    //minio
    let db_server = "http://localhost:9000".parse::<BaseUrl>().unwrap();
    dbg!("Connecting to MinIO at: `{:?}`", &db_server);

    let static_provider = StaticProvider::new("minioadmin", "minioadmin", None);

    let client = ClientBuilder::new(db_server.clone())
        .provider(Some(Box::new(static_provider)))
        .ignore_cert_check(Some(true))
        .build()
        .unwrap();

    let bucket_name = "neelabucket";
    let args = BucketExistsArgs::new(bucket_name);

    let resp = client.bucket_exists(&args.unwrap()).await.unwrap();
    if !resp {
        let mkbucket = MakeBucketArgs::new(bucket_name).unwrap();
        client.make_bucket(&mkbucket).await.unwrap();
    };

    let client_data = web::Data::new(client);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(wasm_actor.clone()))
            .app_data(client_data.clone())
            .wrap(Cors::permissive())
            .wrap(middleware::Logger::default())
            .service(test)
            .service(execute_fn)
            .service(upload_fn)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
