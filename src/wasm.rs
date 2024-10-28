use wasmi::{Engine, Linker, Module, Store};

pub fn run_test(wasm: &[u8], ident: &str, seed: u64) -> Result<u64, anyhow::Error> {
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm)?;
    let mut store = Store::new(&engine, ());
    let linker = Linker::new(&engine);
    let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;
    let test = instance.get_typed_func::<u64, u64>(&mut store, ident)?;
    let result = test.call(&mut store, seed)?;
    log::info!("Test result: {}", result);
    Ok(result)
}
