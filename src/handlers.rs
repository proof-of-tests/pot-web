use std::sync::Arc;

use axum::response::IntoResponse;

use axum::extract::Multipart;

use axum::Extension;
use http::StatusCode;
use send_wrapper::SendWrapper;
use wasmi::*;
use worker::Env;

// Idempotent WASM uploader
// Proof uploader
//  - Check if proof already exists
//  - Check if proof is valid
//  - Store proof

// Proof associations:
//  - Anonymous
//  - Github ID

// Proof table:
//  - wasm hash
//  - owner
//  - created at
//  - seed
//  - hash
//  - weight
//  - register
//  - registers
//  - count

pub struct AppError(axum::response::Response);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        self.0
        // (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self((StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!(value).to_string()).into_response())
    }
}

// #[axum::debug_handler]
pub async fn validate_handler(mut payload: Multipart) -> impl IntoResponse {
    while let Some(field) = payload.next_field().await.unwrap() {
        if field.name() == Some("file") {
            let data = field.bytes().await.unwrap();
            log::info!("File length: {}", data.len());

            let engine = Engine::default();
            let module = Module::new(&engine, &data).unwrap();
            let mut store = Store::new(&engine, ());
            let linker = Linker::new(&engine);
            let instance = linker
                .instantiate(&mut store, &module)
                .unwrap()
                .start(&mut store)
                .unwrap();
            let test = instance.get_typed_func::<u64, u64>(&mut store, "test").unwrap();
            let result = test.call(&mut store, 42).unwrap();
            log::info!("Test result: {}", result);
        }
    }
    "Hello world"
}

// Idempotent WASM uploader
// Uploads a WASM file to R2, uses the hash as the key
#[axum::debug_handler]
pub async fn upload_wasm_handler(
    Extension(env): Extension<Arc<Env>>,
    mut payload: Multipart,
) -> Result<impl IntoResponse, AppError> {
    SendWrapper::new(async move {
        while let Some(field) = payload.next_field().await? {
            if field.name() == Some("file") {
                let data = field.bytes().await?;
                log::info!("File length: {}", data.len());
                // Calculate the hash of the data
                let hash = {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(&data);
                    format!("{:x}", hasher.finalize())
                };
                let vec = data.to_vec();
                env.bucket("wasm")?.put(&hash, vec).execute().await?;
                return Ok(hash);
            }
        }
        Err(AppError((StatusCode::BAD_REQUEST, "No file found").into_response()))
    })
    .await
}
