use core::str;
use std::io::{self, stdin, Bytes, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::Arc;
use tokio::runtime::Runtime;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::{StdoutStream, WasiCtxBuilder};

pub fn run_wasm_function(addr: String, params: Vec<String>) -> String {
    // Run the WASM function in a separate process to avoid runtime conflicts
    // let rt = Runtime::new().unwrap();
    // let res = rt.block_on(run_wasm_function_direct(addr));

    // match res {
    match run_wasm_in_separate_process(&addr, &params) {
        Ok(output) => output,
        Err(e) => {
            let error_msg = format!("Error running WASM function: {}", e);
            eprintln!("{}", error_msg);
            error_msg
        }
    }
}

fn run_wasm_in_separate_process(addr: &str, params: &[String]) -> io::Result<String> {
    // Create a simple command-line utility that runs the WASM file
    // This runs in a separate process, avoiding the runtime conflict
    let path = format!("./src/cache/{}", addr);
    println!("Running WASM function at path: {}", path);

    let mut command = Command::new("wasmtime");
    command.arg(&path);

    // Add parameters to the command
    for param in params {
        command.arg(param);
    }

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;
    let mut output = String::new();
    if let Some(mut stdout) = child.stdout.take() {
        stdout.read_to_string(&mut output)?;
    }
    print!("hi");
    let mut stderr_output = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        stderr.read_to_string(&mut stderr_output)?;
    }

    let status = child.wait()?;

    if status.success() {
        Ok(output)
    } else {
        if !stderr_output.is_empty() {
            Err(io::Error::new(io::ErrorKind::Other, stderr_output))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("WASM execution failed with code: {:?}", status.code()),
            ))
        }
    }
}

// Keep this as a backup or for non-async contexts
// #[allow(dead_code)]
// pub async fn run_wasm_function_direct(addr: String) -> Result<String, Box<dyn std::error::Error>> {
//     let mut config = Config::new();
//     config.async_support(true);
//     let engine = Engine::new(&config)?;

//     let module = Module::from_file(&engine, format!("./src/cache/{}", addr))?;
//     let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
//     preview1::add_to_linker_async(&mut linker, |t| t)?;
//     let stdout_pipe = MemoryOutputPipe::new(100000);

//     let wasi_ctx = WasiCtxBuilder::new()
//         .stdout(stdout_pipe.clone())
//         .stderr(wasmtime_wasi::stderr())
//         .inherit_env()
//         .build_p1();

//     let mut store = Store::new(&engine, wasi_ctx);
//     // let res = store.data_mut();
//     // let pre = linker.instantiate_pre(&module)?;
//     let pre = linker.instantiate(&mut store, &module).unwrap();

//     // let instance = pre.instantiate(&mut store)?;
//     let instance = pre;
//     let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
//     func.call_async(&mut store, ()).await?;
//     let stdout = stdout_pipe.contents();
//     let res = str::from_utf8(&(stdout))?;
//     Ok(res.to_string())
// }

// pub fn run_wasm_fn(addr: String) -> Result<String, Box<dyn std::error::Error>> {
//     let mut config = Config::default();
//     let engine = Engine::new(&config)?;

//     let module = Module::from_file(&engine, format!("./src/cache/{}", addr))?;
//     let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
//     preview1::add_to_linker_sync(&mut linker, |t| t)?;
//     let stdout_pipe = MemoryOutputPipe::new(100000);

//     let wasi_ctx = WasiCtxBuilder::new()
//         .stdout(stdout_pipe.clone())
//         .stderr(wasmtime_wasi::stderr())
//         .inherit_env()
//         .build_p1();

//     let mut store = Store::new(&engine, wasi_ctx);
//     // let res = store.data_mut();
//     let pre = linker.instantiate_pre(&module)?;

//     let instance = pre.instantiate(&mut store)?;
//     let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
//     func.call(&mut store, ())?;
//     let stdout = stdout_pipe.contents();
//     let res = str::from_utf8(&(stdout))?;
//     Ok(res.to_string())
// }
