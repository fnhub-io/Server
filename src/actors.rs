use crate::wasm_function::run_wasm_function;
use actix::prelude::*;
use futures::{future::OkInto, FutureExt};
use std::{io, pin::pin};
pub struct ExecuteFn {
    pub name: String,
    pub params: Vec<String>,
}

impl Message for ExecuteFn {
    type Result = Result<String, std::io::Error>;
}

pub struct WasmEngineActor {}

impl Actor for WasmEngineActor {
    type Context = Context<Self>;
}

impl Handler<ExecuteFn> for WasmEngineActor {
    type Result = Result<String, std::io::Error>;

    fn handle(&mut self, msg: ExecuteFn, ctx: &mut Context<Self>) -> Self::Result {
        println!("Execution command received for fn {}", msg.name);

        Ok(run_wasm_function(msg.name, msg.params))
    }
}
