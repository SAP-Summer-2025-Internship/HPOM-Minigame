# Rust Multi-threaded Web Server

A simple HTTP web server built in Rust that demonstrates thread pool implementation for concurrent request handling. This project showcases how to build a custom thread pool from scratch and use it to handle multiple HTTP requests simultaneously.

## Features

- **Custom Thread Pool**: Hand-built thread pool implementation with configurable worker threads
- **Concurrent Request Handling**: Multiple requests processed simultaneously without blocking
- **Graceful Shutdown**: Proper cleanup of worker threads when the server stops
- **Error Handling**: Safe thread pool creation with custom error types
- **HTTP Server**: Basic HTTP/1.1 server with static file serving

## Project Structure

```
rustdoc_final/
├── src/
│   ├── main.rs          # HTTP server implementation
│   └── lib.rs           # Thread pool library
├── hello.html           # Default homepage
├── 404.html            # Error page
└── Cargo.toml
```

## Quick Start

1. **Navigate to the project:**
   ```bash
   cd "c:\Users\User\OneDrive\Desktop\Rust stuff\rustdoc_final"
   ```

2. **Run the server:**
   ```bash
   cargo run
   ```

3. **Test the server:**
   - Visit `http://127.0.0.1:7878/` for the homepage
   - Visit `http://127.0.0.1:7878/sleep` to test concurrent handling (5-second delay)
   - Visit any other path for a 404 error

## API Reference

### ThreadPool

The core component that manages worker threads for concurrent task execution.

#### Methods

```rust
// Create a new thread pool (panics if size is 0)
pub fn new(size: usize) -> ThreadPool

// Safe thread pool creation
pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError>

// Execute a closure in a worker thread (non-blocking)
pub fn execute<F>(&self, f: F) 
where F: FnOnce() + Send + 'static
```

#### Example Usage

```rust
use rustdoc_final::ThreadPool;

// Create a thread pool with 4 workers
let pool = ThreadPool::new(4);

// Execute tasks concurrently
pool.execute(|| {
    println!("Task 1 running in worker thread");
});

pool.execute(|| {
    println!("Task 2 running in worker thread");
});

// Using safe creation
match ThreadPool::build(0) {
    Ok(pool) => println!("Pool created successfully"),
    Err(e) => println!("Failed to create pool: {:?}", e),
}
```

## How It Works

### Thread Pool Architecture

1. **Main Thread**: Accepts incoming HTTP connections
2. **Worker Threads**: Process HTTP requests concurrently
3. **Channel Communication**: Jobs are sent from main thread to workers via `mpsc::channel`
4. **Shared Receiver**: All workers share access to the job queue through `Arc<Mutex<Receiver>>`

### Request Flow

```
Client Request → TcpListener → Main Thread → ThreadPool → Worker Thread → Response
                    ↓              ↓           ↓            ↓
                 Accept()      execute()    Channel     handle_connection()
```

### Concurrency Demonstration

Open multiple browser tabs and navigate to:
- Tab 1: `http://127.0.0.1:7878/sleep` (will take 5 seconds)
- Tab 2: `http://127.0.0.1:7878/` (should load immediately)

The second request loads while the first is still processing, proving concurrent execution.

## Key Concepts Demonstrated

### 1. Thread Pool Pattern
- Reuses threads instead of creating new ones for each request
- Limits resource consumption
- Improves performance for I/O-bound operations

### 2. Channel Communication
```rust
let (sender, receiver) = mpsc::channel();
let receiver = Arc::new(Mutex::new(receiver));
```

### 3. Graceful Shutdown
```rust
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take()); // Close channel
        
        for worker in self.workers.drain(..) {
            worker.thread.join().unwrap(); // Wait for workers
        }
    }
}
```

### 4. Error Handling
```rust
#[derive(Debug)]
pub struct PoolCreationError {
    message: String,
}
```

## Performance Characteristics

- **Worker Threads**: 40 (configurable in `main.rs`)
- **Concurrent Requests**: Up to 40 simultaneous requests  
- **Queue Behavior**: Additional requests wait in channel buffer
- **Memory Usage**: Minimal overhead per worker thread

## Browser Connection Limitations

**Important Note**: When testing concurrent request handling in browsers, you may observe sequential behavior rather than true concurrency. This is due to browser connection limitations, **not** server limitations.

### Why This Happens:
- **Chrome/Firefox/Safari** limit concurrent connections per domain (typically 6-8)
- Browsers may reuse existing connections for multiple requests
- HTTP/1.1 connection management can serialize requests on the same connection

### Testing True Concurrency:

**Method 1: Use multiple browsers**
```
Chrome:  http://127.0.0.1:7878/sleep
Firefox: http://127.0.0.1:7878/sleep  
Edge:    http://127.0.0.1:7878/sleep
```

**Method 2: Use curl in separate terminals**
```bash
# Terminal 1:
curl http://127.0.0.1:7878/sleep &

# Terminal 2:
curl http://127.0.0.1:7878/sleep &

# Terminal 3:
curl http://127.0.0.1:7878/sleep &

wait
```

**Expected Results**: With curl or multiple browsers, you should see concurrent "Sleep request started" log messages with identical timestamps, proving the thread pool handles multiple requests simultaneously.

## Limitations

- **No HTTP Parsing**: Basic string matching for routes
- **Static Files Only**: No dynamic content generation
- **No HTTPS**: Plain HTTP only
- **Basic Error Handling**: Limited error responses
- **Browser Testing**: Connection limits may mask true concurrency (see Browser Connection Limitations)

## Educational Value

This project demonstrates:
- Manual thread pool implementation
- Rust ownership and borrowing principles
- Channel-based communication between threads
- TCP socket programming
- HTTP protocol basics
- Concurrent programming patterns

## Future Enhancements

- [ ] HTTP request/response parsing library
- [ ] Dynamic routing system
- [ ] SSL/TLS support
- [ ] Configuration file support
- [ ] Logging and metrics
- [ ] Connection pooling
- [ ] Async/await support

## Dependencies

```toml
[dependencies]
# No external dependencies - uses only std library
```

## License

This project is for educational purposes. Feel free to use and modify as needed.

## Contributing

This is a learning project, but suggestions and improvements are welcome!