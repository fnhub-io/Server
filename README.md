# WebAssembly Serverless Execution Platform

This project implements a serverless execution platform for WebAssembly (WASM) functions with execution metrics tracking.

## Setup Instructions

1. **Install the WebAssembly (WASM) compiler:**

   ```sh
   rustup target add wasm32-wasip1
   ```

2. **Install the latest Wasmtime engine:**

   ```sh
   curl https://wasmtime.dev/install.sh -sSf | bash
   ```

3. **Add your function:**

   Place the function you want to execute into the `src/savedWasmFunctions` folder.

4. **Build and start the server:**

   ```sh
   cargo run
   ```

   After running this command, visit [http://127.0.0.1:8080/](http://127.0.0.1:8080/) to verify that the server is working.

## Using the API

### Execute Functions

Execute a WASM function with optional parameters:

```sh
curl -X POST http://localhost:8080/execute \
     -H "Content-Type: application/json" \
     -d '{
           "fn_name": "add.wasm",
           "params": ["5", "7"]
         }'
```

### Upload Functions

Upload a new WASM function to the server:

```sh
curl -X POST http://localhost:8080/upload \
     -H "Content-Type: multipart/form-data" \
     -F "fn_name=sample2.wasm" \
     -F "wasm_file=@/path/to/your/wasm/file.wasm"
```

### Retrieve Execution Metrics

The platform automatically tracks execution metrics for all WASM functions, including execution time, memory usage, and execution count.

#### Get metrics for all functions:

```sh
curl http://localhost:8080/metrics
```

#### Get metrics for a specific function:

```sh
curl http://localhost:8080/metrics/add.wasm
```

## Metrics Storage

Function metrics are stored in a JSON file (`function_metrics.json`) at the root of the project with the following structure:

```json
{
  "function_name.wasm": {
    "name": "function_name.wasm",
    "memory_used_bytes": 123456,
    "total_execution_time_ms": 42,
    "execution_count": 3
  }
}
```

## Example Functions

The project includes sample WASM functions in the `src/savedWasmFunctions` directory that you can use for testing.

### Example: Sample Function

```sh
curl -X POST http://localhost:8080/execute \
     -H "Content-Type: application/json" \
     -d '{"fn_name": "sample.wasm", "params": []}'
```

Thank you!
