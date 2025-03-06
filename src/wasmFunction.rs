use std::io::{self, stdin, Read};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::sync::Arc;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

pub fn run_wasm_function(addr: &str) -> String {
    // Run the WASM function in a separate process to avoid runtime conflicts
    match run_wasm_in_separate_process(addr) {
        Ok(output) => output,
        Err(e) => {
            let error_msg = format!("Error running WASM function: {}", e);
            eprintln!("{}", error_msg);
            error_msg
        }
    }
}

fn run_wasm_in_separate_process(addr: &str) -> io::Result<String> {
    // Create a simple command-line utility that runs the WASM file
    // This runs in a separate process, avoiding the runtime conflict
    let path = format!("./src/savedWasmFunctions/{}", addr);
    println!("Running WASM function at path: {}", path);

    let mut child = Command::new("wasmtime")
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
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
#[allow(dead_code)]
pub async fn run_wasm_function_direct(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, format!("./src/savedWasmFunctions/{}", addr))?;
    dbg!(&module);
    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
    preview1::add_to_linker_async(&mut linker, |t| t)?;

    let pre = linker.instantiate_pre(&module)?;

    let wasi_ctx = WasiCtxBuilder::new()
        .stdout(wasmtime_wasi::stdout())
        .stderr(wasmtime_wasi::stderr())
        .inherit_env()
        .build_p1();

    let mut store = Store::new(&engine, wasi_ctx);
    let instance = pre.instantiate(&mut store)?;
    let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    func.call(&mut store, ())?;
    Ok(())
}
