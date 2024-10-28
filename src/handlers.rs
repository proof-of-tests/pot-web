use std::sync::Arc;

use anyhow::Context as _;
use axum::response::IntoResponse;

use axum::extract::{Multipart, Query};

use axum::Extension;
use http::StatusCode;
use serde::Deserialize;
use wasmi::*;
use worker::{query, Env};

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
#[worker::send]
pub async fn upload_wasm_handler(
    Extension(env): Extension<Arc<Env>>,
    mut payload: Multipart,
) -> Result<impl IntoResponse, AppError> {
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
}

#[derive(Debug, Deserialize)]
pub struct ProofParams {
    wasm: String,
    seed: u64,
    hash: u64,
}

#[axum::debug_handler]
#[worker::send]
pub async fn upload_proof_handler(
    Extension(env): Extension<Arc<Env>>,
    Query(params): Query<ProofParams>,
) -> Result<impl IntoResponse, AppError> {
    let bucket = env.bucket("wasm").unwrap();
    let wasm_object = bucket
        .get(&params.wasm)
        .execute()
        .await?
        .context("WASM not found")?
        .body()
        .context("R2 object without body")?
        .bytes()
        .await?;
    let result = params.hash; // crate::wasm::run_test(&wasm_object, "test", params.seed).context("Failed to run WASM")?;
                              // check that result == params.hash
    if result != params.hash {
        return Err(AppError((StatusCode::BAD_REQUEST, "Invalid proof").into_response()));
    }
    let d1 = env.d1("pot")?;
    // Insert the proof into the database. If the seed or hash already exist, return a 204. We're only interested in new proofs.
    let ret = query!(
        &d1,
        "INSERT INTO pot (wasm, seed, hash) VALUES (?, ?, ?)",
        &params.wasm,
        params.seed,
        params.hash
    )?
    .run()
    .await
    .map_err(|_| AppError((StatusCode::NO_CONTENT, String::default()).into_response()))?;
    log::info!("D1 result: {:?} {:?}", ret.success(), ret.error());
    Ok(StatusCode::CREATED)
    // Fields: wasm, created_at, seed, hash, owner
}
