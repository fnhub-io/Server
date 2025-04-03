use core::str;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::time::Instant;
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// Struct to store function execution metrics
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionMetrics {
    pub name: String,
    pub memory_used_bytes: u64,
    pub total_execution_time_ms: u64,
    pub execution_count: u64,
}

impl FunctionMetrics {
    pub fn new(name: String) -> Self {
        FunctionMetrics {
            name,
            memory_used_bytes: 0,
            total_execution_time_ms: 0,
            execution_count: 0,
        }
    }

    pub fn update(&mut self, memory_bytes: u64, execution_time_ms: u64) {
        self.memory_used_bytes = memory_bytes; // Store the latest memory usage
        self.total_execution_time_ms += execution_time_ms;
        self.execution_count += 1;
    }
}

// Functions to manage the metrics storage
fn load_metrics() -> HashMap<String, FunctionMetrics> {
    let metrics_path = "function_metrics.json";
    match fs::read_to_string(metrics_path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_else(|_| HashMap::new())
        }
        Err(_) => HashMap::new(),
    }
}

fn save_metrics(metrics: &HashMap<String, FunctionMetrics>) -> io::Result<()> {
    let metrics_path = "function_metrics.json";
    let json = serde_json::to_string_pretty(metrics)?;
    fs::write(metrics_path, json)
}

pub fn run_wasm_function(addr: String, params: Vec<String>) -> String {
    // Start timing the execution
    let start_time = Instant::now();
    
    // Run the WASM function in a separate process to avoid runtime conflicts
    let result = match run_wasm_in_separate_process(&addr, &params) {
        Ok(output) => output,
        Err(e) => {
            let error_msg = format!("Error running WASM function: {}", e);
            eprintln!("{}", error_msg);
            error_msg
        }
    };
    
    // Calculate the execution time
    let execution_time_ms = start_time.elapsed().as_millis() as u64;
    
    // Record the metrics
    let fn_name = addr.split('/').last().unwrap_or(&addr).to_string();
    update_function_metrics(fn_name, execution_time_ms);
    
    result
}

fn update_function_metrics(fn_name: String, execution_time_ms: u64) {
    // Estimate memory usage - this is a rough approximation
    // In a real system, you might want to use a more accurate method
    let memory_used = estimate_memory_usage(&fn_name).unwrap_or(0);
    
    // Load existing metrics
    let mut metrics = load_metrics();
    
    // Update metrics for this function
    let fn_metrics = metrics
        .entry(fn_name.clone())
        .or_insert_with(|| FunctionMetrics::new(fn_name));
    
    fn_metrics.update(memory_used, execution_time_ms);
    
    // Save updated metrics
    if let Err(e) = save_metrics(&metrics) {
        eprintln!("Failed to save function metrics: {}", e);
    }
}

fn estimate_memory_usage(fn_name: &str) -> io::Result<u64> {
    // Get file size as a rough estimation of memory usage
    let path = format!("./src/savedWasmFunctions/{}", fn_name);
    match fs::metadata(&path) {
        Ok(metadata) => Ok(metadata.len()),
        Err(e) => {
            eprintln!("Failed to get file size for {}: {}", path, e);
            Err(e)
        }
    }
}

fn run_wasm_in_separate_process(addr: &str, params: &[String]) -> io::Result<String> {
    // Create a simple command-line utility that runs the WASM file
    // This runs in a separate process, avoiding the runtime conflict
    let path = format!("./src/cache/{}", addr);
    println!("Running WASM function at path: {}", path);

    // Check if the file exists
    if !std::path::Path::new(&path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("WASM file not found at path: {}", path),
        ));
    }

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
    
    let mut stderr_output = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        stderr.read_to_string(&mut stderr_output)?;
    }

    let status = child.wait()?;

    if status.success() {
        Ok(output)
    } else {
        if !stderr_output.is_empty() {
            Err(io::Error::new(
                io::ErrorKind::Other, 
                format!("WASM execution error for '{}': {}", addr, stderr_output)
            ))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("WASM execution '{}' failed with code: {:?}", addr, status.code()),
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
