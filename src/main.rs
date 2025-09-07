use sysinfo::{System, SystemExt, ProcessExt};
use std::fs;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use chrono::Local;
use std::collections::HashMap;

const CONFIG_FILE: &str = "MetaGuard.ini";
const VCRUNTIME_URL: &str = "https://aka.ms/vs/17/release/vc_redist.x64.exe";

fn main() {
    println!("================================");
    println!("   MetaGuard - MT4/MT5 監視    ");
    println!("   INI設定版 v0.2.0            ");
    println!("================================\n");

    // 初回起動チェック
    if !std::path::Path::new(CONFIG_FILE).exists() {
        println!("初回起動を検出しました。");
        first_run_setup();
    }

    let mut config = load_or_create_config();
    
    // 自動起動設定をチェックして同期
    sync_auto_start_setting(&config);
    
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
            "9" => open_config_file(),
            "0" => {
                println!("\nプログラムを終了します...");
                break;
            },
            _ => {
                println!("無効な選択です。Enterキーを押してください...");
                wait_for_enter();
            }
        }
        
        // 設定ファイルを再読み込み（外部で編集された場合に対応）
        config = load_or_create_config();
    }
}

fn first_run_setup() {
    println!("\n=== 初回セットアップ ===");
    
    // Visual C++ ランタイムチェック
    println!("\n1. Visual C++ ランタイムをチェック中...");
    if !check_vcruntime_installed() {
        println!("   Visual C++ ランタイムがインストールされていません。");
        print!("   今すぐインストールしますか？ (y/n): ");
        io::stdout().flush().unwrap();
        
        let input = get_user_input();
        if input.trim().to_lowercase() == "y" {
            install_vcruntime();
        } else {
            println!("   スキップしました。後で手動でインストールしてください。");
        }
    } else {
        println!("   ✓ Visual C++ ランタイムは既にインストールされています。");
    }
    
    // 設定ファイル作成
    println!("\n2. 設定ファイルを作成中...");
    let mut config: HashMap<String, String> = HashMap::new();
    
    // MT4/MT5を自動検索
    println!("\n3. MT4/MT5を自動検索中...");
    let instances = auto_search_mt4();
    
    // 設定ファイルに書き込み
    save_initial_config(&instances);
    
    println!("\n✓ 初回セットアップが完了しました！");
    println!("\n設定ファイル '{}' が作成されました。", CONFIG_FILE);
    println!("メモ帳などのテキストエディタで編集できます。");
    
    println!("\nEnterキーを押して続行...");
    wait_for_enter();
}

fn check_vcruntime_installed() -> bool {
    // レジストリをチェック
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        
        let paths = vec![
            r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64",
            r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x86",
            r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64",
        ];
        
        for path in paths {
            if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path) {
                if let Ok(installed) = hklm.get_value::<u32, _>("Installed") {
                    if installed == 1 {
                        return true;
                    }
                }
            }
        }
    }
    
    // DLLファイルの存在確認
    let dll_paths = vec![
        r"C:\Windows\System32\VCRUNTIME140.dll",
        r"C:\Windows\SysWOW64\VCRUNTIME140.dll",
    ];
    
    for path in dll_paths {
        if std::path::Path::new(path).exists() {
            return true;
        }
    }
    
    false
}

fn install_vcruntime() {
    println!("   Visual C++ ランタイムのインストーラーをダウンロード中...");
    
    // PowerShellを使用してダウンロード
    let ps_command = format!(
        "Start-BitsTransfer -Source '{}' -Destination 'vc_redist.x64.exe'",
        VCRUNTIME_URL
    );
    
    match std::process::Command::new("powershell")
        .args(&["-Command", &ps_command])
        .status()
    {
        Ok(status) if status.success() => {
            println!("   ダウンロード完了。インストーラーを起動します...");
            
            // インストーラーを実行
            match std::process::Command::new("vc_redist.x64.exe")
                .arg("/install")
                .arg("/passive")
                .arg("/norestart")
                .status()
            {
                Ok(_) => {
                    println!("   ✓ インストールが完了しました。");
                    // 一時ファイルを削除
                    let _ = fs::remove_file("vc_redist.x64.exe");
                }
                Err(e) => {
                    println!("   インストーラーの実行に失敗: {}", e);
                }
            }
        }
        _ => {
            println!("   ダウンロードに失敗しました。");
            println!("   手動でインストールしてください: {}", VCRUNTIME_URL);
        }
    }
}

fn auto_search_mt4() -> Vec<(String, String)> {
    let mut instances = Vec::new();
    
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
                            instances.push((name.clone(), terminal_path.to_string_lossy().to_string()));
                            println!("   ✓ 発見: {}", name);
                        }
                        
                        if terminal64_path.exists() {
                            let name = format!("{} - {} (64bit)", 
                                path.file_name().unwrap().to_string_lossy(),
                                platform_type
                            );
                            instances.push((name.clone(), terminal64_path.to_string_lossy().to_string()));
                            println!("   ✓ 発見: {}", name);
                        }
                    }
                }
            }
        }
    }
    
    instances
}

fn save_initial_config(instances: &Vec<(String, String)>) {
    let mut content = String::new();
    
    // ヘッダー
    content.push_str(";============================================================\n");
    content.push_str(";          MetaGuard 設定ファイル (MetaGuard.ini)\n");
    content.push_str(";============================================================\n");
    content.push_str(";\n");
    content.push_str("; ■ このファイルの編集方法\n");
    content.push_str(";   1. メモ帳などのテキストエディタで開く\n");
    content.push_str(";   2. 設定値を変更（= の右側の値を編集）\n");
    content.push_str(";   3. ファイルを保存\n");
    content.push_str(";   4. MetaGuardを再起動または設定を再読み込み\n");
    content.push_str(";\n");
    content.push_str("; ■ 注意事項\n");
    content.push_str(";   - 行頭の「;」はコメント行です（この説明文）\n");
    content.push_str(";   - 設定値は「=」の右側に記入します\n");
    content.push_str(";   - パスにはバックスラッシュを2つ使用（例: C:\\\\Program Files\\\\）\n");
    content.push_str(";\n");
    content.push_str(";============================================================\n\n");
    
    // 基本設定セクション
    content.push_str(";------------------------------------------------------------\n");
    content.push_str("; ▼ 基本設定\n");
    content.push_str(";------------------------------------------------------------\n");
    content.push_str("[Settings]\n\n");
    
    content.push_str("; ● MT4/MT5の監視間隔（単位：秒）\n");
    content.push_str(";   設定可能範囲: 10～300秒\n");
    content.push_str(";   推奨値:\n");
    content.push_str(";     10  = 頻繁にチェック（システム負荷：高）\n");
    content.push_str(";     30  = 標準設定（推奨）\n");
    content.push_str(";     60  = ゆっくりチェック（システム負荷：低）\n");
    content.push_str("CheckInterval=30\n\n");
    
    content.push_str("; ● Windows起動時の自動実行\n");
    content.push_str(";   ON  = Windows起動時に自動でMetaGuardを起動\n");
    content.push_str(";   OFF = 手動で起動\n");
    content.push_str("AutoStart=ON\n\n");
    
    // MT4/MT5セクション
    content.push_str(";------------------------------------------------------------\n");
    content.push_str("; ▼ MT4/MT5 監視対象リスト\n");
    content.push_str(";------------------------------------------------------------\n");
    content.push_str("[MT4_MT5]\n\n");
    
    content.push_str("; ● 記入形式\n");
    content.push_str(";   MT_番号=監視|表示名|実行ファイルのフルパス\n");
    content.push_str(";\n");
    content.push_str("; ● 各項目の説明\n");
    content.push_str(";   監視    : ON=監視する、OFF=監視しない\n");
    content.push_str(";   表示名  : MetaGuardで表示される名前（自由に設定可）\n");
    content.push_str(";   パス    : terminal.exe または terminal64.exe のフルパス\n");
    content.push_str(";\n");
    content.push_str("; ● 記入例\n");
    content.push_str(";   MT_1=ON|XM本番口座|C:\\\\Program Files\\\\XM MT4\\\\terminal.exe\n");
    content.push_str(";   MT_2=ON|楽天証券MT4|C:\\\\Program Files\\\\RakutenMT4\\\\terminal64.exe\n");
    content.push_str(";   MT_3=OFF|デモ口座（停止中）|D:\\\\MT4_Demo\\\\terminal.exe\n");
    content.push_str(";\n\n");
    
    for (i, (name, path)) in instances.iter().enumerate() {
        content.push_str(&format!("MT_{}=ON|{}|{}\n", i + 1, name, path));
    }
    
    if instances.is_empty() {
        content.push_str("; 【注意】自動検索でMT4/MT5が見つかりませんでした\n");
        content.push_str(";  以下の例を参考に手動で追加してください：\n");
        content.push_str(";\n");
        content.push_str("; MT_1=ON|表示したい名前|実行ファイルのフルパス\n");
        content.push_str(";\n");
        content.push_str("; 例：\n");
        content.push_str("; MT_1=ON|MetaTrader 4|C:\\\\Program Files\\\\MetaTrader 4\\\\terminal.exe\n");
        content.push_str("; MT_2=ON|MetaTrader 5|C:\\\\Program Files\\\\MetaTrader 5\\\\terminal64.exe\n");
    }
    
    content.push_str("\n");
    content.push_str(";============================================================\n");
    content.push_str("; 設定ファイル終了\n");
    content.push_str(";============================================================\n");
    
    if let Err(e) = fs::write(CONFIG_FILE, content) {
        eprintln!("設定ファイルの作成に失敗: {}", e);
    }
}

fn load_or_create_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    
    if let Ok(contents) = fs::read_to_string(CONFIG_FILE) {
        for line in contents.lines() {
            let line = line.trim();
            
            // コメントと空行をスキップ
            if line.starts_with(';') || line.starts_with('[') || line.is_empty() {
                continue;
            }
            
            // key=value の形式をパース
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let mut value = line[pos + 1..].trim().to_string();
                
                // ON/OFF形式と1/0形式の両方をサポート（互換性維持）
                if key == "AutoStart" {
                    value = match value.to_uppercase().as_str() {
                        "ON" | "TRUE" | "1" => "1".to_string(),
                        "OFF" | "FALSE" | "0" => "0".to_string(),
                        _ => "1".to_string()
                    };
                } else if key.starts_with("MT_") {
                    // MT_エントリの形式を変換
                    let parts: Vec<&str> = value.split('|').collect();
                    if parts.len() == 3 {
                        let enabled = match parts[0].to_uppercase().as_str() {
                            "ON" | "TRUE" | "1" => "1",
                            "OFF" | "FALSE" | "0" => "0",
                            _ => "1"
                        };
                        value = format!("{}|{}|{}", enabled, parts[1], parts[2]);
                    }
                }
                
                config.insert(key, value);
            }
        }
    } else {
        // デフォルト設定
        config.insert("CheckInterval".to_string(), "30".to_string());
        config.insert("AutoStart".to_string(), "1".to_string());
    }
    
    config
}

fn save_config(config: &HashMap<String, String>) {
    let mut content = String::new();
    
    // ヘッダー
    content.push_str(";============================================================\n");
    content.push_str(";          MetaGuard 設定ファイル (MetaGuard.ini)\n");
    content.push_str(";============================================================\n");
    content.push_str(";\n");
    content.push_str("; ■ 編集方法: メモ帳で開いて編集 → 保存 → MetaGuard再起動\n");
    content.push_str(";\n\n");
    
    // 基本設定
    content.push_str(";------------------------------------------------------------\n");
    content.push_str("; ▼ 基本設定\n");
    content.push_str(";------------------------------------------------------------\n");
    content.push_str("[Settings]\n\n");
    
    content.push_str("; ● 監視間隔（秒）: 10～300\n");
    content.push_str(&format!("CheckInterval={}\n\n", 
        config.get("CheckInterval").unwrap_or(&"30".to_string())
    ));
    
    // AutoStartの値をON/OFF形式で保存
    content.push_str("; ● Windows自動起動: ON/OFF\n");
    let auto_start = config.get("AutoStart").unwrap_or(&"1".to_string());
    let auto_start_value = if auto_start == "1" { "ON" } else { "OFF" };
    content.push_str(&format!("AutoStart={}\n\n", auto_start_value));
    
    // MT4/MT5設定
    content.push_str(";------------------------------------------------------------\n");
    content.push_str("; ▼ MT4/MT5 監視対象リスト\n");
    content.push_str(";------------------------------------------------------------\n");
    content.push_str("[MT4_MT5]\n\n");
    
    content.push_str("; 形式: MT_番号=監視|表示名|実行ファイルパス\n");
    content.push_str("; 監視: ON=監視する、OFF=監視しない\n\n");
    
    let mut mt_entries: Vec<_> = config.iter()
        .filter(|(k, _)| k.starts_with("MT_"))
        .collect();
    mt_entries.sort_by_key(|(k, _)| k.as_str());
    
    for (key, value) in mt_entries {
        // MT_エントリの形式をON/OFF形式で保存
        let parts: Vec<&str> = value.split('|').collect();
        if parts.len() == 3 {
            let enabled = if parts[0] == "1" { "ON" } else { "OFF" };
            content.push_str(&format!("{}={}|{}|{}\n", key, enabled, parts[1], parts[2]));
        } else {
            content.push_str(&format!("{}={}\n", key, value));
        }
    }
    
    let _ = fs::write(CONFIG_FILE, content);
}

fn open_config_file() {
    println!("\n設定ファイルを開いています...");
    
    #[cfg(windows)]
    {
        match std::process::Command::new("notepad.exe")
            .arg(CONFIG_FILE)
            .spawn()
        {
            Ok(_) => {
                println!("設定ファイルを開きました。");
                println!("編集後は保存して、このプログラムに戻ってください。");
            }
            Err(e) => {
                println!("エラー: {}", e);
            }
        }
    }
    
    println!("\nEnterキーを押して戻る...");
    wait_for_enter();
}

fn clear_screen() {
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
    println!("9. 設定ファイルを開く");
    println!("0. 終了");
    println!();
    print!("選択してください (0-9): ");
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

fn monitoring_mode(config: &HashMap<String, String>) {
    clear_screen();
    println!("=== 監視モード ===");
    println!("Ctrl+C で停止します\n");
    
    ctrlc::set_handler(move || {
        println!("\n\n監視を停止しました。");
        std::process::exit(0);
    }).expect("Ctrl+Cハンドラーの設定に失敗");
    
    let interval = config.get("CheckInterval")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30);
    
    loop {
        println!("\n[{}] チェック開始...", Local::now().format("%H:%M:%S"));
        check_and_restart_mt4(config);
        
        println!("\n次回チェック: {}秒後", interval);
        for i in (1..=interval).rev() {
            print!("\r残り: {}秒  ", i);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn auto_monitoring_mode(config: &HashMap<String, String>) {
    let interval = config.get("CheckInterval")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30);
    
    loop {
        check_and_restart_mt4(config);
        thread::sleep(Duration::from_secs(interval));
    }
}

fn check_and_restart_mt4(config: &HashMap<String, String>) {
    let mut system = System::new_all();
    system.refresh_processes();
    
    let mt_entries: Vec<_> = config.iter()
        .filter(|(k, _)| k.starts_with("MT_"))
        .collect();
    
    for (_, value) in mt_entries {
        let parts: Vec<&str> = value.split('|').collect();
        if parts.len() != 3 {
            continue;
        }
        
        let enabled = parts[0] == "1";
        let name = parts[1];
        let path = parts[2];
        
        if !enabled {
            continue;
        }
        
        let is_running = system.processes().values().any(|process| {
            let exe_path = process.exe();
            exe_path.to_string_lossy().to_lowercase() == path.to_lowercase()
        });
        
        if is_running {
            println!("✓ {} - 実行中", name);
        } else {
            println!("✗ {} - 停止中", name);
            
            match std::process::Command::new(path).spawn() {
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

fn list_mt4_instances(config: &HashMap<String, String>) {
    clear_screen();
    println!("=== MT4/MT5 一覧 ===\n");
    
    let mut mt_entries: Vec<_> = config.iter()
        .filter(|(k, _)| k.starts_with("MT_"))
        .collect();
    
    if mt_entries.is_empty() {
        println!("登録されているMT4/MT5はありません。");
    } else {
        mt_entries.sort_by_key(|(k, _)| k.as_str());
        
        for (i, (_, value)) in mt_entries.iter().enumerate() {
            let parts: Vec<&str> = value.split('|').collect();
            if parts.len() == 3 {
                let enabled = parts[0] == "1";
                let name = parts[1];
                let path = parts[2];
                
                println!("{}. [{}] {}", 
                    i + 1,
                    if enabled { "有効" } else { "無効" },
                    name
                );
                println!("   パス: {}", path);
                println!();
            }
        }
    }
    
    println!("\nEnterキーを押して戻る...");
    wait_for_enter();
}

fn add_mt4_instance(config: &mut HashMap<String, String>) {
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
    
    if !std::path::Path::new(&path).exists() {
        println!("指定されたファイルが見つかりません: {}", path);
        wait_for_enter();
        return;
    }
    
    // 次の番号を決定
    let mut next_num = 1;
    loop {
        let key = format!("MT_{}", next_num);
        if !config.contains_key(&key) {
            break;
        }
        next_num += 1;
    }
    
    let key = format!("MT_{}", next_num);
    let value = format!("1|{}|{}", name, path);
    config.insert(key, value);
    
    save_config(config);
    println!("\n✓ MT4/MT5を追加しました！");
    thread::sleep(Duration::from_secs(2));
}

fn toggle_mt4_instance(config: &mut HashMap<String, String>) {
    clear_screen();
    println!("=== 有効/無効の切り替え ===\n");
    
    let mut mt_entries: Vec<_> = config.iter()
        .filter(|(k, _)| k.starts_with("MT_"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    
    if mt_entries.is_empty() {
        println!("登録されているMT4/MT5はありません。");
        wait_for_enter();
        return;
    }
    
    mt_entries.sort_by_key(|(k, _)| k.clone());
    
    for (i, (_, value)) in mt_entries.iter().enumerate() {
        let parts: Vec<&str> = value.split('|').collect();
        if parts.len() == 3 {
            let enabled = parts[0] == "1";
            let name = parts[1];
            
            println!("{}. [{}] {}", 
                i + 1,
                if enabled { "有効" } else { "無効" },
                name
            );
        }
    }
    
    print!("\n切り替える番号を入力してください (0で戻る): ");
    io::stdout().flush().unwrap();
    let input = get_user_input();
    
    if let Ok(num) = input.trim().parse::<usize>() {
        if num == 0 {
            return;
        }
        if num > 0 && num <= mt_entries.len() {
            let (key, value) = &mt_entries[num - 1];
            let parts: Vec<&str> = value.split('|').collect();
            
            if parts.len() == 3 {
                let enabled = parts[0] != "1";
                let new_value = format!("{}|{}|{}", 
                    if enabled { "1" } else { "0" },
                    parts[1],
                    parts[2]
                );
                
                config.insert(key.clone(), new_value);
                save_config(config);
                
                let status = if enabled { "有効" } else { "無効" };
                println!("\n✓ {} を{}にしました", parts[1], status);
                thread::sleep(Duration::from_secs(2));
            }
        } else {
            println!("無効な番号です。");
            wait_for_enter();
        }
    }
}

fn remove_mt4_instance(config: &mut HashMap<String, String>) {
    clear_screen();
    println!("=== MT4/MT5を削除 ===\n");
    
    let mut mt_entries: Vec<_> = config.iter()
        .filter(|(k, _)| k.starts_with("MT_"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    
    if mt_entries.is_empty() {
        println!("登録されているMT4/MT5はありません。");
        wait_for_enter();
        return;
    }
    
    mt_entries.sort_by_key(|(k, _)| k.clone());
    
    for (i, (_, value)) in mt_entries.iter().enumerate() {
        let parts: Vec<&str> = value.split('|').collect();
        if parts.len() >= 2 {
            println!("{}. {}", i + 1, parts[1]);
        }
    }
    
    print!("\n削除する番号を入力してください (0で戻る): ");
    io::stdout().flush().unwrap();
    let input = get_user_input();
    
    if let Ok(num) = input.trim().parse::<usize>() {
        if num == 0 {
            return;
        }
        if num > 0 && num <= mt_entries.len() {
            let (key, value) = &mt_entries[num - 1];
            let parts: Vec<&str> = value.split('|').collect();
            let name = if parts.len() >= 2 { parts[1] } else { "不明" };
            
            config.remove(key);
            save_config(config);
            
            println!("\n✓ {} を削除しました", name);
            thread::sleep(Duration::from_secs(2));
        } else {
            println!("無効な番号です。");
            wait_for_enter();
        }
    }
}

fn search_and_add_mt4(config: &mut HashMap<String, String>) {
    println!("\nMT4/MT5を検索中...");
    
    let instances = auto_search_mt4();
    let mut added_count = 0;
    
    for (name, path) in instances {
        // 既に登録されているかチェック
        let already_exists = config.values().any(|v| {
            let parts: Vec<&str> = v.split('|').collect();
            parts.len() == 3 && parts[2] == path
        });
        
        if !already_exists {
            let mut next_num = 1;
            loop {
                let key = format!("MT_{}", next_num);
                if !config.contains_key(&key) {
                    break;
                }
                next_num += 1;
            }
            
            let key = format!("MT_{}", next_num);
            let value = format!("1|{}|{}", name, path);
            config.insert(key, value);
            added_count += 1;
        }
    }
    
    if added_count > 0 {
        println!("\n{}個の新しいMT4/MT5を追加しました", added_count);
    } else {
        println!("\n新しいMT4/MT5は見つかりませんでした");
    }
    
    thread::sleep(Duration::from_secs(3));
}

fn change_check_interval(config: &mut HashMap<String, String>) {
    clear_screen();
    println!("=== チェック間隔の変更 ===\n");
    
    let current = config.get("CheckInterval")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(30);
    
    println!("現在の間隔: {}秒", current);
    
    print!("\n新しい間隔を入力してください (10-300秒): ");
    io::stdout().flush().unwrap();
    let input = get_user_input();
    
    if let Ok(interval) = input.trim().parse::<u64>() {
        if interval >= 10 && interval <= 300 {
            config.insert("CheckInterval".to_string(), interval.to_string());
            save_config(config);
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
    
    let mut config = load_or_create_config();
    let is_enabled = config.get("AutoStart").map(|v| v == "1").unwrap_or(true);
    
    println!("現在の状態: {}", if is_enabled { "有効" } else { "無効" });
    
    if is_enabled {
        print!("\n自動起動を無効にしますか？ (y/n): ");
        io::stdout().flush().unwrap();
        let input = get_user_input();
        
        if input.trim().to_lowercase() == "y" {
            config.insert("AutoStart".to_string(), "0".to_string());
            save_config(&config);
            
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
            config.insert("AutoStart".to_string(), "1".to_string());
            save_config(&config);
            
            if let Err(e) = setup_auto_start() {
                println!("エラー: {}", e);
            } else {
                println!("\n✓ 自動起動を有効にしました");
            }
        }
    }
    
    thread::sleep(Duration::from_secs(2));
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

fn sync_auto_start_setting(config: &HashMap<String, String>) {
    let config_auto_start = config.get("AutoStart").map(|v| v == "1").unwrap_or(true);
    let registry_auto_start = check_auto_start_enabled();
    
    // 設定ファイルとレジストリが一致しない場合、設定ファイルの値に合わせる
    if config_auto_start != registry_auto_start {
        if config_auto_start {
            println!("設定ファイルに従って自動起動を有効化します...");
            if let Err(e) = setup_auto_start() {
                eprintln!("自動起動設定エラー: {}", e);
            } else {
                println!("✓ 自動起動を有効にしました\n");
            }
        } else {
            println!("設定ファイルに従って自動起動を無効化します...");
            if let Err(e) = remove_auto_start() {
                eprintln!("自動起動解除エラー: {}", e);
            } else {
                println!("✓ 自動起動を無効にしました\n");
            }
        }
    }
}