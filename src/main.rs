use eframe::egui;
use sysinfo::{System, SystemExt, ProcessExt};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{Duration, Instant};

// カラーテーマ構造体
struct ColorTheme {
    text_primary: egui::Color32,
    text_secondary: egui::Color32,
    surface1: egui::Color32,
    surface2: egui::Color32,
    primary: egui::Color32,
    danger: egui::Color32,
    secondary: egui::Color32,
    neutral: egui::Color32,
    border: egui::Color32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    check_interval_seconds: u64,
    mt4_instances: Vec<Mt4Instance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Mt4Instance {
    name: String,
    path: String,
    enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            check_interval_seconds: 30,
            mt4_instances: vec![],  // 空のベクタでスタート
        }
    }
}

struct Mt4MonitorApp {
    config: Config,
    monitoring_active: bool,
    logs: Vec<String>,
    new_mt4_name: String,
    new_mt4_path: String,
    last_check: Option<Instant>,
    auto_start_enabled: bool,
    dark_mode: bool,
}

impl Default for Mt4MonitorApp {
    fn default() -> Self {
        let config = load_or_create_config();
        let mut auto_start_enabled = check_auto_start_enabled();
        
        // 初回起動時（自動起動が設定されていない場合）は自動的に有効化
        if !auto_start_enabled {
            if let Err(e) = setup_auto_start() {
                eprintln!("自動起動の設定に失敗しました: {}", e);
            } else {
                auto_start_enabled = true;
                println!("自動起動を有効にしました");
            }
        }
        
        let mut app = Self {
            config,
            monitoring_active: false,
            logs: vec![format!("[{}] プログラムを起動しました", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))],
            new_mt4_name: String::new(),
            new_mt4_path: String::new(),
            last_check: None,
            auto_start_enabled,
            dark_mode: true,  // デフォルトはダークモード
        };
        
        // 初回起動時（MT4/MT5が登録されていない場合）は自動検索を実行
        if app.config.mt4_instances.is_empty() || 
           (app.config.mt4_instances.len() == 1 && 
            app.config.mt4_instances[0].name == "MT4 メイン口座" &&
            app.config.mt4_instances[0].path == r"C:\Program Files (x86)\MetaTrader 4\terminal.exe") {
            app.add_log("初回起動のため、MT4/MT5を自動検索します...");
            app.search_mt4_installations();
            // 検索結果を保存
            save_config(&app.config);
        }
        
        app
    }
}

impl eframe::App for Mt4MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ===== カラーテーマの定義 =====
        let theme = if self.dark_mode {
            // ダークモードの色定義（Material Design 3準拠）
            ColorTheme {
                text_primary: egui::Color32::from_gray(222),   // 87% white
                text_secondary: egui::Color32::from_gray(158),  // 60% white
                surface1: egui::Color32::from_gray(30),        // 1dp elevation
                surface2: egui::Color32::from_gray(46),        // 8dp elevation
                primary: egui::Color32::from_rgb(129, 199, 132),    // Material Green 300
                danger: egui::Color32::from_rgb(239, 154, 154),     // Material Red 300
                secondary: egui::Color32::from_rgb(144, 202, 249),  // Material Blue 300
                neutral: egui::Color32::from_gray(97),              // Surface 8dp
                border: egui::Color32::from_gray(60),
            }
        } else {
            // ライトモードの色定義
            ColorTheme {
                text_primary: egui::Color32::from_gray(30),
                text_secondary: egui::Color32::from_gray(80),
                surface1: egui::Color32::WHITE,
                surface2: egui::Color32::from_gray(245),
                primary: egui::Color32::from_rgb(76, 175, 80),      // Material Green 500
                danger: egui::Color32::from_rgb(244, 67, 54),       // Material Red 500
                secondary: egui::Color32::from_rgb(33, 150, 243),   // Material Blue 500
                neutral: egui::Color32::from_gray(158),             // Material Grey 500
                border: egui::Color32::from_gray(200),
            }
        };
        
        // ビジュアルテーマの適用
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            // ヘッダーとモード切り替え
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("MetaGuard - MT4/MT5 監視ツール").color(theme.text_primary));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(if self.dark_mode { "☀ ライトモード" } else { "🌙 ダークモード" }).clicked() {
                        self.dark_mode = !self.dark_mode;
                    }
                });
            });
            ui.separator();
            
            // 監視ボタン（改善された配色）
            ui.horizontal(|ui| {
                if self.monitoring_active {
                    if ui.add_sized([150.0, 40.0], 
                        egui::Button::new("⏸ 監視を停止")
                            .fill(theme.danger))
                        .clicked() {
                        self.monitoring_active = false;
                        self.add_log("監視を停止しました");
                    }
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("🟢 監視中").size(20.0).color(theme.primary));
                } else {
                    if ui.add_sized([150.0, 40.0], 
                        egui::Button::new("▶ 監視を開始")
                            .fill(theme.primary))
                        .clicked() {
                        self.monitoring_active = true;
                        self.add_log("監視を開始しました");
                    }
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("🔴 停止中").size(20.0).color(theme.danger));
                }
            });
            
            // 監視処理
            if self.monitoring_active {
                if self.last_check.is_none() {
                    self.last_check = Some(Instant::now());
                }
                
                if let Some(last) = self.last_check {
                    if last.elapsed().as_secs() >= self.config.check_interval_seconds {
                        self.check_mt4_instances();
                        self.last_check = Some(Instant::now());
                    }
                }
            }
            
            ui.separator();
            
            // チェック間隔
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("チェック間隔（秒）:").color(theme.text_primary));
                let mut interval = self.config.check_interval_seconds as f32;
                ui.add(egui::Slider::new(&mut interval, 10.0..=300.0));
                self.config.check_interval_seconds = interval as u64;
            });
            
            ui.separator();
            
            // MT4/MT5一覧
            ui.label(egui::RichText::new("MT4/MT5一覧").size(20.0).color(theme.text_primary));
            
            // 一覧を枠で囲む（改善されたサーフェス色）
            egui::Frame::none()
                .fill(theme.surface1)
                .stroke(egui::Stroke::new(1.0, theme.border))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    // スクロールエリアで一覧を表示
                    egui::ScrollArea::vertical()
                        .id_source("mt4_list_scroll")
                        .max_height(200.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let mut remove_indices = vec![];
                            
                            for (i, instance) in self.config.mt4_instances.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut instance.enabled, "");
                                    
                                    // 名前を固定幅で表示
                                    ui.add_sized([180.0, 18.0], 
                                        egui::Label::new(
                                            egui::RichText::new(&instance.name).color(theme.text_primary)
                                        )
                                    );
                                    
                                    // パスを残りのスペースで表示
                                    let available_width = ui.available_width() - 60.0;
                                    ui.add_sized([available_width, 18.0], 
                                        egui::Label::new(
                                            egui::RichText::new(&instance.path).color(theme.text_secondary)
                                        ).truncate(true).wrap(false)
                                    );
                                    
                                    if ui.small_button("削除").clicked() {
                                        remove_indices.push(i);
                                    }
                                });
                                ui.add_space(2.0);
                            }
                            
                            for i in remove_indices.iter().rev() {
                                self.config.mt4_instances.remove(*i);
                            }
                        });
                });
            
            ui.add_space(10.0);
            
            // 新規追加
            ui.label(egui::RichText::new("MT4/MT5を追加").size(20.0).color(theme.text_primary));
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("名前:").color(theme.text_primary));
                ui.text_edit_singleline(&mut self.new_mt4_name);
            });
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("パス:").color(theme.text_primary));
                ui.add(egui::TextEdit::singleline(&mut self.new_mt4_path).desired_width(400.0));
                if ui.button("参照").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("実行ファイル", &["exe"])
                        .pick_file() {
                        self.new_mt4_path = path.display().to_string();
                    }
                }
                if ui.add(
                    egui::Button::new("🔍 MT4/MT5を自動検索")
                        .fill(theme.secondary)
                ).clicked() {
                    self.search_mt4_installations();
                }
            });
            
            if ui.add_sized([100.0, 30.0], 
                egui::Button::new("追加")
                    .fill(theme.primary)
            ).clicked() && !self.new_mt4_name.is_empty() {
                self.config.mt4_instances.push(Mt4Instance {
                    name: self.new_mt4_name.clone(),
                    path: self.new_mt4_path.clone(),
                    enabled: true,
                });
                self.new_mt4_name.clear();
                self.new_mt4_path.clear();
            }
            
            ui.separator();
            
            // 保存ボタンと自動起動
            ui.horizontal(|ui| {
                if ui.add_sized([120.0, 35.0], 
                    egui::Button::new("💾 設定を保存")
                        .fill(theme.neutral)
                ).clicked() {
                    save_config(&self.config);
                    self.add_log("設定を保存しました");
                }
                
                ui.add_space(20.0);
                
                // 自動起動のチェックボックス
                let checkbox_color = if self.auto_start_enabled { 
                    theme.primary 
                } else { 
                    theme.text_secondary
                };
                
                if ui.checkbox(&mut self.auto_start_enabled, 
                    egui::RichText::new("Windows起動時に自動実行").color(checkbox_color)
                ).clicked() {
                    if self.auto_start_enabled {
                        if let Err(e) = setup_auto_start() {
                            self.add_log(&format!("自動起動設定エラー: {}", e));
                            self.auto_start_enabled = false;
                        } else {
                            self.add_log("自動起動を有効にしました");
                        }
                    } else {
                        if let Err(e) = remove_auto_start() {
                            self.add_log(&format!("自動起動解除エラー: {}", e));
                            self.auto_start_enabled = true;
                        } else {
                            self.add_log("自動起動を無効にしました");
                        }
                    }
                }
            });
            
            ui.separator();
            
            // ログ
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("ログ").size(20.0).color(theme.text_primary));
                if ui.small_button("📄 ログを保存").clicked() {
                    self.save_logs();
                }
                if ui.small_button("🗑 ログをクリア").clicked() {
                    self.logs.clear();
                    self.add_log("ログをクリアしました");
                }
            });
            
            // ログ表示エリア（改善されたサーフェス色）
            egui::Frame::none()
                .fill(theme.surface2)
                .stroke(egui::Stroke::new(1.0, theme.border))
                .inner_margin(egui::Margin::same(4.0))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_source("log_scroll")
                        .max_height(150.0)
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let mut log_text = String::new();
                            for log in self.logs.iter().rev().take(100) {
                                log_text.push_str(log);
                                log_text.push('\n');
                            }
                            ui.add(egui::TextEdit::multiline(&mut log_text)
                                .font(egui::TextStyle::Small)
                                .desired_rows(8)
                                .interactive(true)
                                .text_color(theme.text_primary)
                                .desired_width(f32::INFINITY));
                        });
                });
        });
        
        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

impl Mt4MonitorApp {
    fn add_log(&mut self, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("[{}] {}", timestamp, message);
        self.logs.push(log_entry);
        
        // ログが多すぎる場合は古いものを削除
        if self.logs.len() > 500 {
            self.logs.remove(0);
        }
    }
    
    fn check_mt4_instances(&mut self) {
        let mut system = System::new_all();
        system.refresh_processes();
        
        // configをクローンして借用問題を回避
        let instances = self.config.mt4_instances.clone();
        
        for instance in &instances {
            if !instance.enabled {
                continue;
            }
            
            let is_running = system.processes().values().any(|process| {
                let exe_path = process.exe();
                exe_path.to_string_lossy().to_lowercase() == instance.path.to_lowercase()
            });
            
            if !is_running {
                self.add_log(&format!("⚠️ {} が停止しています", instance.name));
                
                match std::process::Command::new(&instance.path).spawn() {
                    Ok(_) => {
                        self.add_log(&format!("✅ {} を起動しました", instance.name));
                    }
                    Err(e) => {
                        self.add_log(&format!("❌ {} の起動に失敗: {}", instance.name, e));
                    }
                }
            }
        }
    }
    
    fn search_mt4_installations(&mut self) {
        self.add_log("MT4/MT5を検索中...");
        
        // ユーザーディレクトリを先に計算
        let user_appdata = format!(r"C:\Users\{}\AppData\Roaming", std::env::var("USERNAME").unwrap_or_default());
        
        let search_paths = vec![
            r"C:\Program Files (x86)",
            r"C:\Program Files",
            r"D:\Program Files (x86)",
            r"D:\Program Files",
            // ユーザーディレクトリも検索
            &user_appdata,
        ];
        
        let mut found_terminals = Vec::new();
        
        for base_path in search_paths {
            if let Ok(entries) = fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().unwrap().to_string_lossy().to_lowercase();
                        
                        // MT4/MT5のディレクトリパターンをチェック
                        if dir_name.contains("metatrader") || dir_name.contains("mt4") || dir_name.contains("mt5") {
                            // terminal.exe と terminal64.exe をチェック
                            let terminal_path = path.join("terminal.exe");
                            let terminal64_path = path.join("terminal64.exe");
                            
                            let platform_type = if dir_name.contains("mt5") || dir_name.contains("metatrader 5") {
                                "MT5"
                            } else {
                                "MT4"
                            };
                            
                            if terminal_path.exists() {
                                let name = format!("{} - {} (32bit)", 
                                    path.file_name().unwrap().to_string_lossy(),
                                    platform_type
                                );
                                found_terminals.push((name, terminal_path.to_string_lossy().to_string()));
                            }
                            
                            if terminal64_path.exists() {
                                let name = format!("{} - {} (64bit)", 
                                    path.file_name().unwrap().to_string_lossy(),
                                    platform_type
                                );
                                found_terminals.push((name, terminal64_path.to_string_lossy().to_string()));
                            }
                        }
                    }
                }
            }
        }
        
        // 重複を削除
        found_terminals.sort_by(|a, b| a.1.cmp(&b.1));
        found_terminals.dedup_by(|a, b| a.1 == b.1);
        
        if found_terminals.is_empty() {
            self.add_log("MT4/MT5が見つかりませんでした");
        } else {
            self.add_log(&format!("{}個のMT4/MT5を検出しました", found_terminals.len()));
            
            // 既存の一覧と重複しないものだけを追加
            let mut added_count = 0;
            for (name, path) in found_terminals {
                // 既に一覧に存在するかチェック
                let already_exists = self.config.mt4_instances.iter().any(|instance| {
                    instance.path.to_lowercase() == path.to_lowercase()
                });
                
                if !already_exists {
                    self.config.mt4_instances.push(Mt4Instance {
                        name: name.clone(),
                        path: path.clone(),
                        enabled: true,
                    });
                    self.add_log(&format!("✅ 「{}」を追加しました", name));
                    added_count += 1;
                } else {
                    self.add_log(&format!("  既に登録済み: {}", name));
                }
            }
            
            if added_count > 0 {
                self.add_log(&format!("{}個の新しいMT4/MT5を一覧に追加しました", added_count));
            } else {
                self.add_log("新しいMT4/MT5は見つかりませんでした（すべて登録済み）");
            }
        }
    }
    
    fn save_logs(&self) {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("metaguard_log_{}.txt", timestamp);
        
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(&filename)
            .add_filter("テキストファイル", &["txt"])
            .save_file() {
            
            let mut content = String::new();
            content.push_str("MetaGuard ログファイル\n");
            content.push_str(&format!("保存日時: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
            content.push_str(&"=".repeat(50));
            content.push_str("\n\n");
            
            // ログを古い順に保存
            for log in self.logs.iter() {
                content.push_str(log);
                content.push_str("\n");
            }
            
            if let Err(e) = fs::write(&path, content) {
                eprintln!("ログ保存エラー: {}", e);
            }
        }
    }
}

fn load_or_create_config() -> Config {
    if let Ok(contents) = fs::read_to_string("metaguard_config.toml") {
        if let Ok(config) = toml::from_str(&contents) {
            return config;
        }
    }
    Config::default()
}

fn save_config(config: &Config) {
    if let Ok(toml) = toml::to_string_pretty(config) {
        let _ = fs::write("metaguard_config.toml", toml);
    }
}

fn setup_auto_start() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let (key, _) = hkcu.create_subkey(path)?;
        
        let exe_path = std::env::current_exe()?;
        key.set_value("MetaGuard", &exe_path.to_string_lossy().to_string())?;
    }
    
    Ok(())
}

fn remove_auto_start() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let key = hkcu.open_subkey_with_flags(path, KEY_ALL_ACCESS)?;
        
        key.delete_value("MetaGuard")?;
    }
    
    Ok(())
}

fn check_auto_start_enabled() -> bool {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") {
            if let Ok(_) = hkcu.get_value::<String, _>("MetaGuard") {
                return true;
            }
        }
    }
    
    false
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 日本語フォントを追加
    fonts.font_data.insert(
        "japanese".to_owned(),
        egui::FontData::from_static(include_bytes!("../fonts/NotoSansJP-VariableFont_wght.ttf")),
    );
    
    // 日本語フォントを最優先に設定
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "japanese".to_owned());
    
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "japanese".to_owned());
    
    ctx.set_fonts(fonts);
    
    // UIのスタイルを設定
    let mut style = (*ctx.style()).clone();
    
    // テキストスタイルのサイズ調整
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::new(24.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Body, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Button, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Small, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
        (egui::TextStyle::Monospace, egui::FontId::new(14.0, egui::FontFamily::Monospace)),
    ].into();
    
    // スペーシングも調整
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.indent = 20.0;
    
    ctx.set_style(style);
}

fn main() {
    println!("MetaGuard を起動しています...");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    match eframe::run_native(
        "MetaGuard",
        options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(Mt4MonitorApp::default())
        }),
    ) {
        Ok(_) => println!("アプリケーションを終了しました"),
        Err(e) => eprintln!("エラー: {:?}", e),
    }
}