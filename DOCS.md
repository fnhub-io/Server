# Orbit Platform - WebAssembly Serverless System

A serverless platform that executes WebAssembly (WASM) functions on demand.

## Project Structure

- **Backend Server**: Rust-based server that handles function storage and execution
- **CLI Tool**: Command-line utility for deploying WASM functions to the platform

## Setup Requirements

### Prerequisites

- Rust toolchain
- MinIO server (for function storage)
- Wasmtime (for WASM execution)

### Backend Setup

1. **Install the WebAssembly compiler target:**
   ```sh
   rustup target add wasm32-wasip1
   ```

2. **Install Wasmtime engine:**
   ```sh
   curl https://wasmtime.dev/install.sh -sSf | bash
   ```

3. **Start MinIO server:**
   Ensure your MinIO server is running locally on port 9000 with default credentials (`minioadmin`/`minioadmin`).

4. **Build and start the server:**
   ```sh
   cargo run
   ```
   
   The server will be available at `http://127.0.0.1:8080/`

### CLI Setup

From the CLI directory:

```sh
cargo build --release
```

## Usage

### Deploy a function

Navigate to your Rust project directory and use the CLI:

```sh
orbit-cli
```

The CLI will:
1. Compile your Rust project to WebAssembly
2. Upload the WASM file to the Orbit platform

### Execute a function

```sh
curl -X POST http://localhost:8080/execute \
     -H "Content-Type: application/json" \
     -d '{
           "fn_name": "your_function_name",
           "params": ["param1", "param2"]
         }'
```

## API Endpoints

- `GET /`: Health check endpoint
- `POST /execute`: Execute a WASM function
- `POST /upload`: Upload a new WASM function

## Project Components

- `main.rs`: Server initialization and MinIO connection setup
- `actors.rs`: Actix actors for WASM function execution
- `routes.rs`: API endpoints for function upload and execution
- `wasm_function.rs`: WASM execution engine

## License

[Your project license here]
