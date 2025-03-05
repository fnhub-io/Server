use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use aws_sdk_s3::config::{Credentials, Region, Config};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use http::Uri;

pub struct MinioClient {
    client: Client,
    bucket: String,
}

impl MinioClient {
    pub async fn new(
        endpoint: &str, 
        access_key: &str, 
        secret_key: &str, 
        bucket_name: &str,
        region: Option<&str>
    ) -> Result<Self, Box<dyn Error>> {
        let uri: Uri = endpoint.parse()?;

        let credentials = Credentials::new(
            access_key, 
            secret_key, 
            None, 
            None,
            "static"
        );

        let region = region.unwrap_or("us-east-1").to_string();

        let config = S3ConfigBuilder::new()
            .region(Region::new(region.clone()))
            .endpoint_url(endpoint)
            .credentials_provider(credentials)
            .build();

        let client = Client::from_conf(config);

        Ok(MinioClient { 
            client, 
            bucket: bucket_name.to_string() 
        })
    }

    
    pub async fn store_wasm_file(&self, file_path: &str, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        self.client.put_object()
            .bucket(&self.bucket)
            .key(file_name)
            .body(contents.into())
            .send()
            .await?;

        Ok(())
    }

    
    pub async fn retrieve_wasm_file(&self, file_name: &str, local_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let result = self.client.get_object()
            .bucket(&self.bucket)
            .key(file_name)
            .send()
            .await?;

        let mut file = File::create(local_path)?;
        let mut stream = result.body.collect().await?.into_bytes();
        std::io::copy(&mut stream.as_ref(), &mut file)?;

        Ok(())
    }
}


pub async fn create_minio_client() -> Result<MinioClient, Box<dyn std::error::Error>> {
    let endpoint = "http://localhost:9000".to_string();
    let access_key = "minioadmin".to_string();
    let secret_key = "minioadmin".to_string();
    let bucket_name = "wasm-functions".to_string();

    MinioClient::new(&endpoint, &access_key, &secret_key, &bucket_name, None).await
}
