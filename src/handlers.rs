use axum::response::IntoResponse;

use axum::extract::Multipart;

use wasmi::*;

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
