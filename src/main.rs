use sysinfo::{System, SystemExt, ProcessExt};
use serde::{Deserialize, Serialize};
use std::fs;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use chrono::Local;

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
            mt4_instances: vec![],
        }
    }
}

fn main() {
    println!("================================");
    println!("   MetaGuard - MT4/MT5 監視    ");
    println!("   コンソール版 v0.1.0         ");
    println!("================================\n");

    // 自動起動設定をチェック
    if !check_auto_start_enabled() {
        println!("Windows起動時の自動実行を設定します...");
        if let Err(e) = setup_auto_start() {
            eprintln!("自動起動設定エラー: {}", e);
        } else {
            println!("✓ 自動起動を有効にしました\n");
        }
    }

    let mut config = load_or_create_config();
    
    // 初回起動時は自動検索
    if config.mt4_instances.is_empty() {
        println!("初回起動のため、MT4/MT5を自動検索します...");
        search_and_add_mt4(&mut config);
        save_config(&config);
    }

    // 自動監視モードで起動（引数がある場合）
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--auto" {
        println!("自動監視モードで起動しました");
        auto_monitoring_mode(&config);
        return;
    }

    // メインループ
    loop {
        clear_screen();
        show_header();
        show_menu();

        let choice = get_user_input();
        
        match choice.trim() {
            "1" => monitoring_mode(&config),
            "2" => list_mt4_instances(&config),
            "3" => add_mt4_instance(&mut config),
            "4" => toggle_mt4_instance(&mut config),
            "5" => remove_mt4_instance(&mut config),
            "6" => {
                search_and_add_mt4(&mut config);
                save_config(&config);
            },
            "7" => {
                change_check_interval(&mut config);
                save_config(&config);
            },
            "8" => toggle_auto_start(),
            "9" => {
                println!("\nプログラムを終了します...");
                break;
            },
            _ => {
                println!("無効な選択です。Enterキーを押してください...");
                wait_for_enter();
            }
        }
    }
}

fn clear_screen() {
    // Windowsのコンソールをクリア
    std::process::Command::new("cmd")
        .args(&["/C", "cls"])
        .status()
        .unwrap();
}

fn show_header() {
    println!("================================");
    println!("   MetaGuard - MT4/MT5 監視    ");
    println!("================================");
    println!("現在時刻: {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
    println!();
}

fn show_menu() {
    println!("メインメニュー:");
    println!("1. 監視を開始");
    println!("2. MT4/MT5一覧を表示");
    println!("3. MT4/MT5を追加");
    println!("4. MT4/MT5の有効/無効を切り替え");
    println!("5. MT4/MT5を削除");
    println!("6. MT4/MT5を自動検索");
    println!("7. チェック間隔を変更");
    println!("8. Windows自動起動設定");
    println!("9. 終了");
    println!();
    print!("選択してください (1-9): ");
    io::stdout().flush().unwrap();
}

fn get_user_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

fn wait_for_enter() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

fn monitoring_mode(config: &Config) {
    clear_screen();
    println!("=== 監視モード ===");
    println!("Ctrl+C で停止します\n");
    
    // Ctrl+Cハンドラーを設定
    ctrlc::set_handler(move || {
        println!("\n\n監視を停止しました。");
        std::process::exit(0);
    }).expect("Ctrl+Cハンドラーの設定に失敗");
    
    loop {
        println!("\n[{}] チェック開始...", Local::now().format("%H:%M:%S"));
        check_and_restart_mt4(config);
        
        println!("\n次回チェック: {}秒後", config.check_interval_seconds);
        for i in (1..=config.check_interval_seconds).rev() {
            print!("\r残り: {}秒  ", i);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn auto_monitoring_mode(config: &Config) {
    loop {
        check_and_restart_mt4(config);
        thread::sleep(Duration::from_secs(config.check_interval_seconds));
    }
}

fn check_and_restart_mt4(config: &Config) {
    let mut system = System::new_all();
    system.refresh_processes();
    
    for instance in &config.mt4_instances {
        if !instance.enabled {
            continue;
        }
        
        let is_running = system.processes().values().any(|process| {
            let exe_path = process.exe();
            exe_path.to_string_lossy().to_lowercase() == instance.path.to_lowercase()
        });
        
        if is_running {
            println!("✓ {} - 実行中", instance.name);
        } else {
            println!("✗ {} - 停止中", instance.name);
            
            match std::process::Command::new(&instance.path).spawn() {
                Ok(_) => {
                    println!("  → 起動しました！");
                }
                Err(e) => {
                    println!("  → 起動失敗: {}", e);
                }
            }
        }
    }
}

fn list_mt4_instances(config: &Config) {
    clear_screen();
    println!("=== MT4/MT5 一覧 ===\n");
    
    if config.mt4_instances.is_empty() {
        println!("登録されているMT4/MT5はありません。");
    } else {
        for (i, instance) in config.mt4_instances.iter().enumerate() {
            println!("{}. [{}] {}", 
                i + 1,
                if instance.enabled { "有効" } else { "無効" },
                instance.name
            );
            println!("   パス: {}", instance.path);
            println!();
        }
    }
    
    println!("\nEnterキーを押して戻る...");
    wait_for_enter();
}

fn add_mt4_instance(config: &mut Config) {
    clear_screen();
    println!("=== MT4/MT5を追加 ===\n");
    
    print!("名前を入力してください: ");
    io::stdout().flush().unwrap();
    let name = get_user_input().trim().to_string();
    
    if name.is_empty() {
        println!("名前が入力されていません。");
        wait_for_enter();
        return;
    }
    
    print!("実行ファイルのパスを入力してください: ");
    io::stdout().flush().unwrap();
    let path = get_user_input().trim().to_string();
    
    if path.is_empty() {
        println!("パスが入力されていません。");
        wait_for_enter();
        return;
    }
    
    // パスの存在確認
    if !std::path::Path::new(&path).exists() {
        println!("指定されたファイルが見つかりません: {}", path);
        wait_for_enter();
        return;
    }
    
    config.mt4_instances.push(Mt4Instance {
        name,
        path,
        enabled: true,
    });
    
    save_config(config);
    println!("\n✓ MT4/MT5を追加しました！");
    thread::sleep(Duration::from_secs(2));
}

fn toggle_mt4_instance(config: &mut Config) {
    clear_screen();
    println!("=== 有効/無効の切り替え ===\n");
    
    if config.mt4_instances.is_empty() {
        println!("登録されているMT4/MT5はありません。");
        wait_for_enter();
        return;
    }
    
    for (i, instance) in config.mt4_instances.iter().enumerate() {
        println!("{}. [{}] {}", 
            i + 1,
            if instance.enabled { "有効" } else { "無効" },
            instance.name
        );
    }
    
    print!("\n切り替える番号を入力してください (0で戻る): ");
    io::stdout().flush().unwrap();
    let input = get_user_input();
    
    if let Ok(num) = input.trim().parse::<usize>() {
        if num == 0 {
            return;
        }
        if num > 0 && num <= config.mt4_instances.len() {
            config.mt4_instances[num - 1].enabled = !config.mt4_instances[num - 1].enabled;
            save_config(config);
            
            let status = if config.mt4_instances[num - 1].enabled { "有効" } else { "無効" };
            println!("\n✓ {} を{}にしました", config.mt4_instances[num - 1].name, status);
            thread::sleep(Duration::from_secs(2));
        } else {
            println!("無効な番号です。");
            wait_for_enter();
        }
    }
}

fn remove_mt4_instance(config: &mut Config) {
    clear_screen();
    println!("=== MT4/MT5を削除 ===\n");
    
    if config.mt4_instances.is_empty() {
        println!("登録されているMT4/MT5はありません。");
        wait_for_enter();
        return;
    }
    
    for (i, instance) in config.mt4_instances.iter().enumerate() {
        println!("{}. {}", i + 1, instance.name);
    }
    
    print!("\n削除する番号を入力してください (0で戻る): ");
    io::stdout().flush().unwrap();
    let input = get_user_input();
    
    if let Ok(num) = input.trim().parse::<usize>() {
        if num == 0 {
            return;
        }
        if num > 0 && num <= config.mt4_instances.len() {
            let removed = config.mt4_instances.remove(num - 1);
            save_config(config);
            
            println!("\n✓ {} を削除しました", removed.name);
            thread::sleep(Duration::from_secs(2));
        } else {
            println!("無効な番号です。");
            wait_for_enter();
        }
    }
}

fn search_and_add_mt4(config: &mut Config) {
    println!("\nMT4/MT5を検索中...");
    
    let user_appdata = format!(r"C:\Users\{}\AppData\Roaming", 
        std::env::var("USERNAME").unwrap_or_default()
    );
    
    let search_paths = vec![
        r"C:\Program Files (x86)",
        r"C:\Program Files",
        r"D:\Program Files (x86)",
        r"D:\Program Files",
        &user_appdata,
    ];
    
    let mut found_count = 0;
    
    for base_path in search_paths {
        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap().to_string_lossy().to_lowercase();
                    
                    if dir_name.contains("metatrader") || dir_name.contains("mt4") || dir_name.contains("mt5") {
                        let terminal_path = path.join("terminal.exe");
                        let terminal64_path = path.join("terminal64.exe");
                        
                        let platform_type = if dir_name.contains("mt5") { "MT5" } else { "MT4" };
                        
                        if terminal_path.exists() {
                            let name = format!("{} - {} (32bit)", 
                                path.file_name().unwrap().to_string_lossy(),
                                platform_type
                            );
                            let path_str = terminal_path.to_string_lossy().to_string();
                            
                            if !config.mt4_instances.iter().any(|i| i.path == path_str) {
                                config.mt4_instances.push(Mt4Instance {
                                    name: name.clone(),
                                    path: path_str,
                                    enabled: true,
                                });
                                println!("✓ 発見: {}", name);
                                found_count += 1;
                            }
                        }
                        
                        if terminal64_path.exists() {
                            let name = format!("{} - {} (64bit)", 
                                path.file_name().unwrap().to_string_lossy(),
                                platform_type
                            );
                            let path_str = terminal64_path.to_string_lossy().to_string();
                            
                            if !config.mt4_instances.iter().any(|i| i.path == path_str) {
                                config.mt4_instances.push(Mt4Instance {
                                    name: name.clone(),
                                    path: path_str,
                                    enabled: true,
                                });
                                println!("✓ 発見: {}", name);
                                found_count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    
    if found_count > 0 {
        println!("\n{}個の新しいMT4/MT5を追加しました", found_count);
    } else {
        println!("\n新しいMT4/MT5は見つかりませんでした");
    }
    
    thread::sleep(Duration::from_secs(3));
}

fn change_check_interval(config: &mut Config) {
    clear_screen();
    println!("=== チェック間隔の変更 ===\n");
    println!("現在の間隔: {}秒", config.check_interval_seconds);
    
    print!("\n新しい間隔を入力してください (10-300秒): ");
    io::stdout().flush().unwrap();
    let input = get_user_input();
    
    if let Ok(interval) = input.trim().parse::<u64>() {
        if interval >= 10 && interval <= 300 {
            config.check_interval_seconds = interval;
            println!("\n✓ チェック間隔を{}秒に変更しました", interval);
            thread::sleep(Duration::from_secs(2));
        } else {
            println!("10〜300の間で入力してください。");
            wait_for_enter();
        }
    } else {
        println!("無効な入力です。");
        wait_for_enter();
    }
}

fn toggle_auto_start() {
    clear_screen();
    println!("=== Windows自動起動設定 ===\n");
    
    let is_enabled = check_auto_start_enabled();
    println!("現在の状態: {}", if is_enabled { "有効" } else { "無効" });
    
    if is_enabled {
        print!("\n自動起動を無効にしますか？ (y/n): ");
        io::stdout().flush().unwrap();
        let input = get_user_input();
        
        if input.trim().to_lowercase() == "y" {
            if let Err(e) = remove_auto_start() {
                println!("エラー: {}", e);
            } else {
                println!("\n✓ 自動起動を無効にしました");
            }
        }
    } else {
        print!("\n自動起動を有効にしますか？ (y/n): ");
        io::stdout().flush().unwrap();
        let input = get_user_input();
        
        if input.trim().to_lowercase() == "y" {
            if let Err(e) = setup_auto_start() {
                println!("エラー: {}", e);
            } else {
                println!("\n✓ 自動起動を有効にしました");
            }
        }
    }
    
    thread::sleep(Duration::from_secs(2));
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
        let auto_start_command = format!("{} --auto", exe_path.to_string_lossy());
        key.set_value("MetaGuard", &auto_start_command)?;
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