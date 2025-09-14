# MetaGuard セキュアインストーラービルドスクリプト v2.0
# このスクリプトはRustプログラムをビルドし、セキュリティ警告を軽減するインストーラーを作成します

Write-Host "================================================" -ForegroundColor Cyan
Write-Host "     MetaGuard インストーラービルダー v2.0        " -ForegroundColor Cyan
Write-Host "  (セキュリティ警告対策版)                     " -ForegroundColor Cyan
Write-Host "================================================" -ForegroundColor Cyan
Write-Host ""

# 1. 必要なディレクトリを作成
Write-Host "[1/8] ディレクトリを準備中..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path "installer" | Out-Null
New-Item -ItemType Directory -Force -Path "resources" | Out-Null

# 2. マニフェストファイルをコピー
Write-Host "[2/8] マニフェストファイルを準備中..." -ForegroundColor Yellow
if (Test-Path "MetaGuard.exe.manifest") {
    Copy-Item "MetaGuard.exe.manifest" "resources\" -Force
    Write-Host "  ✓ マニフェストファイルをコピーしました" -ForegroundColor Green
} else {
    Write-Host "  ! マニフェストファイルが見つかりません" -ForegroundColor Yellow
}

# 3. Rustプログラムをリリースビルド（静的リンク）
Write-Host "[3/8] Rustプログラムをビルド中（静的リンク）..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "  ✗ ビルドに失敗しました" -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ ビルドが完了しました（VC++ランタイム統合済み）" -ForegroundColor Green

# 4. サンプル設定ファイルを作成
Write-Host "[4/8] サンプル設定ファイルを作成中..." -ForegroundColor Yellow

$sampleConfig = @"
;============================================================
;          MetaGuard 設定ファイル (MetaGuard.ini)
;============================================================
;
; ■ このファイルの編集方法
;   1. メモ帳などのテキストエディタで開く
;   2. 設定値を変更（= の右側の値を編集）
;   3. ファイルを保存
;   4. MetaGuardを再起動
;
; ■ セキュリティについて
;   - このプログラムはVC++ランタイムを内蔵しているため
;     追加のDLLやランタイムのインストールは不要です
;
;============================================================

[Settings]

; ● MT4/MT5の監視間隔（単位：秒）
;   推奨値: 30秒
CheckInterval=30

; ● Windows起動時の自動実行
;   ON  = 自動起動する
;   OFF = 手動起動
AutoStart=OFF

[MT4_MT5]

; ● 記入形式
;   MT_番号=監視|表示名|実行ファイルのフルパス
;
; ● 例：
; MT_1=ON|XM MT4|C:\\Program Files\\XM MT4\\terminal.exe
; MT_2=ON|楽天証券MT4|C:\\Program Files\\RakutenMT4\\terminal64.exe
;
"@

$sampleConfig | Out-File -Encoding UTF8 "MetaGuard.ini.sample"
Write-Host "  ✓ サンプル設定ファイルを作成しました" -ForegroundColor Green

# 5. アイコンファイルを準備（簡易版）
Write-Host "[5/8] アイコンファイルを準備中..." -ForegroundColor Yellow
if (-not (Test-Path "icon.ico")) {
    Write-Host "  ! アイコンファイルが見つかりません" -ForegroundColor Yellow
    Write-Host "    icon.ico を用意することを推奨します" -ForegroundColor Yellow
}

# 6. LICENSEファイルを作成
Write-Host "[6/8] ライセンスファイルを作成中..." -ForegroundColor Yellow

$licenseText = @"
MIT License

Copyright (c) 2024 Haruki Hosono

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"@

$licenseText | Out-File -Encoding UTF8 "LICENSE"
Write-Host "  ✓ ライセンスファイルを作成しました" -ForegroundColor Green

# 7. 実行ファイルのサイズを確認
Write-Host "[7/8] ビルド結果を確認中..." -ForegroundColor Yellow
if (Test-Path "target\release\MetaGuard.exe") {
    $fileInfo = Get-Item "target\release\MetaGuard.exe"
    $sizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
    Write-Host "  ✓ MetaGuard.exe: $sizeMB MB" -ForegroundColor Green
    Write-Host "  ✓ VC++ランタイム統合済み（DLL不要）" -ForegroundColor Green
} else {
    Write-Host "  ✗ ビルドファイルが見つかりません" -ForegroundColor Red
    exit 1
}

# 8. Inno Setupでインストーラーをコンパイル
Write-Host "[8/8] インストーラーを作成中..." -ForegroundColor Yellow

# Inno Setupのパスを検索
$innoSetupPaths = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
)

$isccPath = $null
foreach ($path in $innoSetupPaths) {
    if (Test-Path $path) {
        $isccPath = $path
        break
    }
}

if ($isccPath) {
    Write-Host "  Inno Setupを実行中..." -ForegroundColor Yellow
    & "$isccPath" setup.iss

    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "================================================" -ForegroundColor Green
        Write-Host "        インストーラー作成完了！              " -ForegroundColor Green
        Write-Host "================================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "インストーラーの場所:" -ForegroundColor Cyan
        Write-Host "  installer\MetaGuard_Setup_0.2.0_Installer.exe" -ForegroundColor White
        Write-Host ""
        Write-Host "セキュリティ警告対策:" -ForegroundColor Cyan
        Write-Host "  • インストーラー形式で配布（exe単体より信頼性高）" -ForegroundColor White
        Write-Host "  • VC++ランタイム統合済み（依存DLLなし）" -ForegroundColor White
        Write-Host "  • UAC設定最適化（管理者権限不要）" -ForegroundColor White
        Write-Host ""
        Write-Host "このインストーラーには以下が含まれています:" -ForegroundColor Cyan
        Write-Host "  • MetaGuard.exe (メインプログラム - VC++ランタイム統合済み)" -ForegroundColor White
        Write-Host "  • サンプル設定ファイル" -ForegroundColor White
        Write-Host "  • 自動起動設定オプション" -ForegroundColor White
        Write-Host "  ※ Visual C++ ランタイムのインストールは不要です" -ForegroundColor Green
        Write-Host ""
        Write-Host "さらなるセキュリティ向上のために:" -ForegroundColor Yellow
        Write-Host "  1. コード署名証明書を取得" -ForegroundColor White
        Write-Host "  2. setup.issで署名設定を有効化" -ForegroundColor White
        Write-Host "  3. インストーラーに署名を適用" -ForegroundColor White
    } else {
        Write-Host "  ✗ インストーラーの作成に失敗しました" -ForegroundColor Red
    }
} else {
    Write-Host "  ✗ Inno Setupが見つかりません" -ForegroundColor Red
    Write-Host ""
    Write-Host "Inno Setupをインストールしてください:" -ForegroundColor Yellow
    Write-Host "  https://jrsoftware.org/isdl.php" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "インストール後、このスクリプトを再実行してください。" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Enterキーを押して終了..."
Read-Host