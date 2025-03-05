use crate::wasmFunction::run_wasm_function;
use actix::prelude::*;

pub struct ExecuteFn {
    pub name: String,
}

impl Message for ExecuteFn {
    type Result = Result<String, std::io::Error>;
}

pub struct WasmEngineActor{}

impl Actor for WasmEngineActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteFn> for WasmEngineActor {
    type Result = Result<String, std::io::Error>;

    fn handle(&mut self, msg: ExecuteFn, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Execution command received for fn {}", msg.name);
        let output = run_wasm_function(&msg.name);
        Ok(output)
    }
}