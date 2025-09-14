# MetaGuard

MT4/MT5の監視・自動起動ツール

## 機能

- MT4/MT5の動作状態を監視
- 停止時に自動再起動
- Windows起動時の自動実行
- INI形式の設定ファイル
- 複数のMT4/MT5インスタンスの管理

## インストーラーでのインストール（推奨）

### セキュリティ警告対策版インストーラー

1. [Releases](https://github.com/harukihosono/MetaGuard/releases)から最新のインストーラー（`MetaGuard_Setup_*_Installer.exe`）をダウンロード
2. インストーラーを実行
3. **Visual C++ランタイムは不要**（静的リンク済み）
4. セキュリティ警告が軽減されたインストーラー形式

### インストーラーに含まれるもの

- MetaGuard.exe（メインプログラム - VC++ランタイム統合済み）
- サンプル設定ファイル
- 自動起動設定オプション
- アプリケーションマニフェスト

### セキュリティ対策

- **静的リンク**: Visual C++ランタイムを内蔵（DLL不要）
- **UAC最適化**: 管理者権限不要で実行
- **インストーラー形式**: exe単体より信頼性が高い
- **デジタル署名対応**: 証明書があれば署名可能

## 開発者向け：インストーラーの作成方法

### 必要なツール

1. **Rust** (1.70以降)
   - https://www.rust-lang.org/tools/install

2. **Inno Setup 6**
   - https://jrsoftware.org/isdl.php
   - インストール時に「Inno Setup Preprocessor」も選択

### ビルド手順

```powershell
# 1. リポジトリをクローン
git clone https://github.com/harukihosono/MetaGuard.git
cd MetaGuard

# 2. セキュア版インストーラーをビルド
powershell -ExecutionPolicy Bypass -File build-installer-secure.ps1
```

ビルドが完了すると、`installer\MetaGuard_Setup_*_Installer.exe`にセキュリティ対策済みインストーラーが作成されます。

### デジタル署名（将来的に検討）

現在はコストの関係で署名を行っていませんが、将来的に検討中です。

#### 署名証明書の取得
1. **商用証明書**: DigiCert、Sectigo、GlobalSignなどから購入
2. **EVコード署名**: より高い信頼レベル（推奨）
3. **価格**: 年間30,000円～100,000円程度

#### 署名の設定手順
1. 証明書をインストール
2. `setup.iss`の署名設定を有効化：
   ```ini
   SignTool=signtool /a /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $f
   SignedUninstaller=yes
   ```
3. ビルド時に自動署名される

#### 無料の代替案
- **Self-signed証明書**: 警告は出るが識別は可能
- **GitHub Actions**: CI/CDで自動ビルド・配布
- **VirusTotal**: 事前にスキャンして安全性を証明

## 手動インストール

インストーラーを使用しない場合：

1. **Visual C++ ランタイムは不要**（静的リンク済み）
2. MetaGuard.exeを任意のフォルダに配置
3. 初回起動時に設定ファイル（MetaGuard.ini）が自動作成されます

**注意**: 手動配布の場合、セキュリティ警告が出やすくなります。インストーラー形式を推奨します。

## 使い方

1. MetaGuardを起動
2. 初回起動時は自動でMT4/MT5を検索
3. メニューから「監視を開始」を選択
4. Ctrl+Cで監視を停止

## 設定ファイル

`MetaGuard.ini`で以下の設定が可能：

- 監視間隔（10～300秒）
- Windows自動起動のON/OFF
- MT4/MT5インスタンスの追加/削除/有効化/無効化

## トラブルシューティング

### セキュリティ警告が表示される場合

#### インストーラー版（推奨）
1. **Windows Defender SmartScreen**
   - 「詳細情報」をクリック
   - 「実行」をクリック
   - インストーラー形式のため警告は軽減されます

2. **アンチウイルスソフト**
   - 初回のみ警告が出る場合があります
   - 「許可」または「除外リストに追加」

#### exe単体版
1. **警告が多く出る可能性があります**
2. **対策**: インストーラー版の使用を強く推奨
3. **除外設定**: アンチウイルスソフトでMetaGuard.exeを除外

#### 無料でできる対策

1. **インストーラーに同梱されたガイドを確認**
   - インストール時に「SECURITY_INFO.txt」が表示されます
   - 「INSTALL_GUIDE.txt」に詳細な手順を記載

2. **VirusTotalでの事前スキャン**
   - https://www.virustotal.com でファイルをスキャン
   - 結果をユーザーに提示して信頼性を証明

3. **オープンソースであることをアピール**
   - GitHubでソースコードを公開
   - 記載されたセキュリティ情報で安全性を明示

4. **段階的な信頼性構築**
   - 初期ユーザーがインストールすることでSmartScreenの警告が減る
   - GitHub StarsやIssuesでコミュニティの信頼性を構築

#### 将来的な解決策（収益が出たら）
- **デジタル署名**: コード署名証明書の取得・適用
- **EV証明書**: より高い信頼レベルを得る

### MT4/MT5が起動しない場合

1. パスが正しいか確認
2. 管理者権限で実行してみる
3. **Visual C++ランタイムは不要**（静的リンク済み）
4. MT4/MT5のインストール状態を確認

## ライセンス

MIT License

## 作者

Haruki Hosono

## リポジトリ

https://github.com/harukihosono/MetaGuard