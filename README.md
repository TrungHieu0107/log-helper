# SQL Log Parser - Rust Version

## Building the Project

### Prerequisites

1. **Install Rust** (if not already installed):
   ```powershell
   # Using rustup (recommended)
   winget install Rustlang.Rustup
   # Or download from https://rustup.rs/
   ```

2. **Restart your terminal** after installing Rust.

### Build Steps

```powershell
# Navigate to the Rust project
cd d:\linh_ta_linh_tinh\log-helper\rust

# Build debug version (fast compilation, for development)
cargo build

# Build release version (optimized, for distribution)
cargo build --release

# Run the application
cargo run --release
```

### Output

- **Debug**: `target\debug\sql_log_parser.exe`
- **Release**: `target\release\sql_log_parser.exe`

### Expected Binary Size

| Build Type | Approximate Size |
|------------|------------------|
| Debug      | ~15-20 MB        |
| Release    | ~4-5 MB          |

### Features

The `sql` feature is enabled by default, which includes ODBC support for executing queries against SQL Server.

To build without SQL support:
```powershell
cargo build --release --no-default-features
```

## Project Structure

```
rust/
├── Cargo.toml           # Dependencies and build config
├── src/
│   ├── main.rs          # Entry point
│   ├── app.rs           # eframe::App implementation
│   ├── config/
│   │   └── mod.rs       # Configuration management
│   ├── core/
│   │   ├── mod.rs
│   │   ├── log_parser.rs      # Log file parsing
│   │   ├── sql_formatter.rs   # SQL formatting
│   │   ├── query_processor.rs # Query orchestration
│   │   └── html_generator.rs  # HTML report generation
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── main_window.rs     # Main UI window
│   │   └── theme.rs           # Dark theme
│   └── utils/
│       ├── mod.rs
│       ├── file_helper.rs     # File system utilities
│       ├── encoding.rs        # SHIFT-JIS encoding
│       └── clipboard.rs       # Windows clipboard
```

## Key Differences from C++ Version

| Aspect | C++ | Rust |
|--------|-----|------|
| GUI Framework | ImGui + DirectX11 | egui + native renderer |
| JSON | nlohmann/json | serde_json |
| Encoding | Windows APIs | encoding_rs crate |
| Clipboard | Win32 APIs | clipboard-win crate |
| Error Handling | Exceptions | Result<T, E> |
| Memory | Manual RAII | Ownership system |

## Running Tests

```powershell
cargo test
```
