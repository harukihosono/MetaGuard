// build.rs - ビルド時の設定スクリプト
// Visual C++ランタイムを静的リンクして、DLL依存を完全に排除

fn main() {
    // Windows MSVCターゲットの場合のみ実行
    if cfg!(target_os = "windows") && cfg!(target_env = "msvc") {
        // static_vcruntimeクレートが自動的に処理
        // リリースビルド時にVCランタイムを静的リンク

        // 追加のリンカーフラグ（必要に応じて）
        // コンソールアプリケーションとして設定
        println!("cargo:rustc-link-arg=/SUBSYSTEM:CONSOLE");

        // Windows 10の機能を有効化
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");

        // デバッグ情報を最小化（リリースビルド時）
        #[cfg(not(debug_assertions))]
        {
            println!("cargo:rustc-link-arg=/DEBUG:NONE");
        }
    }
}