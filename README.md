# 🎯 CAND - Colorful And Nice Debugging

> **Beautiful embedded-first Rust logging library for ESP32 to servers with colorful output and zero-panic design.**


## ✨ **Why Choose CAND?**

| **Feature** | **Benefit** | **Use Case** |
| :-- | :-- | :-- |
| 🎨 **Smart Colors** | Spot issues instantly | `logger.log_err("❌ Clear visibility")` |
| ⚡ **no_std Ready** | Runs everywhere | ESP32, STM32, WASM, bare metal |
| 🛡️ **Never Panic** | Production safe | `try_run()` handles all errors gracefully |
| 🔌 **Pluggable** | Your infrastructure | Files, UART, RTT, databases, networks |
| 📦 **Tiny Binary** | <1KB overhead | Perfect for memory-constrained devices |
| 🎯 **2-Line Setup** | Start in seconds | Works out of the box |

## 🚀 **Quick Start**

### **Desktop/Server**

```toml
[dependencies]
cand = "0.2"
```

```rust
use cand::Logger;
use std::time::Instant;

fn main() {
  let mut logger = Logger(Instant::now(), ());
    
  logger.log_ok("🚀 Server started successfully!");
  logger.log_info("📡 Listening on port 8080");
  logger.log_warn("⚠️ High memory usage: 87%");
  logger.log_err("❌ Database connection failed");
}
```


### **Embedded/ESP32 with no_std**

```toml
[dependencies]
cand = { version = "0.2", default-feature=false, features=["colors"] }
```


## 🎨 **Beautiful Output**

CAND automatically color-codes your logs for instant visual feedback:

- **🟢 `log_ok()`** - Success operations (green)
- **🔵 `log_info()`** - Informational messages (blue)
- **🟡 `log_warn()`** - Warnings that need attention (yellow)
- **🔴 `log_err()`** - Critical errors (red)

![sample of output](sample.png)

## 🛡️ **Error Handling That Never Panics**

```rust
use cand::Logger;

fn risky_operation() -> Result<String, &'static str> {
  Err("network timeout")
}

fn fallback_handler(mut logger: Logger<std::time::Instant, ()>) {
  logger.log_warn("🔄 Entering fallback mode");
  logger.log_info("💾 Switching to cached data");
}

fn main() {
  let logger = Logger(std::time::Instant::now(), ());
    
  // Automatic error logging and graceful recovery
  let (data, recovered_logger) = logger.try_run(
    risky_operation(),
    fallback_handler
  );
}


```

### **Embedded UART**

```rust
struct UartStorage{
  // Serial can be from any mcu serial
  serial: Serial 
};

impl StorageProvider for UartStorage {
  fn write_data(&mut self, d: impl ufmt::uDebug) {
    // Write to UART, RTT, or any embedded output
    ufmt::uwrite!(self.serial,"{:?}", d);
  }
}
```


## 🎛️ **Feature Flags**

| **Feature** | **Description** | **Default** |
| :-- | :-- | :-- |
| `std` | Standard library support, enables `Instant` time provider | ✅ |
| `colors` | ANSI color output for beautiful terminal logs | ✅ |
| `ufmt` | Embedded-friendly formatting with zero allocations | No |


## 📊 **Performance**

- **⚡ Zero allocations** with `ufmt` feature
- **🚀 178147+ logs/second** on example benchmark


## 🏗️ **API Reference**

### **Core Types**

```rust
// Make sure Type should TimeProvider or StorageProvider
// Main logger with time and storage providers
pub struct Logger<T: TimeProvider, S: StorageProvider>(pub T, pub S);

```

## 🧪 **Examples**

Check out the [examples](examples/) directory:

- **[`basic_error_handling`](examples/basic_error_handling.rs)** - Error recovery patterns
- **[`sample`](examples/sample.rs)** - Feature showcase and demo
- **[`benchmark`](examples/benchmark.rs)** - Benchmark of 10_000 logs print

Run examples:

```bash
cargo run --example sample
```

## 📄 **License**

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.
