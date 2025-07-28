//! CAND Banner Demo for Terminal Screenshots

use cand::Logger;

fn main() {
    println!(); // Add some top spacing
    showcase_cand();
    println!(); // Add some bottom spacing
}

fn showcase_cand() {
    let mut logger = Logger((), ());

    // 🔥 **Eye-catching header**
    logger.log_ok("🔥 CAND - Colorful And Nice Debugging 🔥");
    logger.log_info("🎯 From ESP32 microcontrollers to production servers");
    println!();

    // ✨ **Core features showcase**
    logger.log_info("✨ Why developers choose CAND:");
    logger.log_ok("  🎨 Smart color coding for instant issue visibility");
    logger.log_ok("  ⚡ no_std native - perfect for ESP32/embedded");
    logger.log_ok("  🛡️  Never-panic error handling with graceful recovery");
    logger.log_ok("  📦 Tiny binary footprint (<1KB overhead)");
    logger.log_ok("  🔌 Pluggable backends (UART, RTT, files, networks)");
    println!();

    // 🚀 **Live demo section**
    logger.log_info("🚀 Live ESP32 Demo:");
    logger.log_info("  🔌 Initializing peripherals...");

    // Add small delays for realistic feel
    std::thread::sleep(std::time::Duration::from_millis(100));
    logger.log_ok("  📡 WiFi stack ready");

    std::thread::sleep(std::time::Duration::from_millis(80));
    logger.log_info("  🌐 Connecting to MQTT broker...");
    logger.log_ok("  📶 Connected: 192.168.1.42");

    // Show all log levels in action
    logger.log_info("  📊 Reading sensors...");
    logger.log_ok("  🌡️  Temperature: 24.5°C");
    logger.log_warn("  ⚠️  Memory usage: 87%");
    logger.log_err("  ❌ I2C timeout on device 0x48");
    println!();

    // 💡 **Error recovery showcase**
    logger.log_info("🛡️  Error Recovery Demo:");
    let sensor_result: Result<f32, &str> = Err("sensor disconnected");

    #[cfg(feature = "std")]
    let (_, mut logger) = logger.try_get(sensor_result, |mut logger| {
        logger.log_warn("  🔄 Auto-recovery initiated");
        logger.log_info("  💾 Switching to backup sensor");
        logger.log_ok("  ✅ Failover complete");
    });

    println!();

    // 🎮 **Getting started**
    logger.log_info("🎮 Quick Start (2 lines):");
    logger.log_ok("  let mut logger = Logger((), ());");
    logger.log_ok("  logger.log_ok(\"🚀 Ready!\");");
    println!();

    // 📦 **Installation**
    logger.log_info("📦 Installation:");
    logger.log_ok("  cargo add cand --features std,colors,ufmt");
    logger.log_info("  # Or for embedded: --no-default-features --features ufmt");
    println!();

    // ✨ **Closing**
    logger.log_ok("✨ Make your debugging colorful and nice with CAND! ✨");
    logger.log_info("📖 https://github.com/CosmoBunny/cand");
}
