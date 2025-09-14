; MetaGuard インストーラー設定ファイル (Inno Setup)
;
; このファイルをInno Setupでコンパイルするとインストーラーが作成されます

#define MyAppName "MetaGuard"
#define MyAppVersion "0.2.0"
#define MyAppPublisher "Haruki Hosono"
#define MyAppURL "https://github.com/harukihosono/MetaGuard"
#define MyAppExeName "MetaGuard.exe"

[Setup]
; 基本設定
AppId={{E5F3D8A2-9C47-4B22-9E33-1A2B3C4D5E6F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppContact=https://github.com/harukihosono/MetaGuard/issues
AppComments=MT4/MT5監視・自動起動ツール - オープンソースソフトウェア
AppCopyright=MIT License - https://github.com/harukihosono/MetaGuard
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes

; 出力設定
OutputDir=installer
OutputBaseFilename=MetaGuard_Setup_{#MyAppVersion}_Installer
; SetupIconFile=icon.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
DisableWelcomePage=no
; InfoBeforeFile=SECURITY_INFO.txt

; 署名設定（デジタル署名を持っている場合）
; コメントを外して署名を有効化
; SignTool=signtool /a /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $f
; SignedUninstaller=yes

; Windows版のみ対応
ArchitecturesInstallIn64BitMode=x64
ArchitecturesAllowed=x64

; 権限設定
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; バージョン情報
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=MT4/MT5監視・自動起動ツール
VersionInfoCopyright=Copyright (C) 2024 {#MyAppPublisher}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 6.1; Check: not IsAdminInstallMode
Name: "autostart"; Description: "Windows起動時に自動的にMetaGuardを起動する"; GroupDescription: "起動オプション:"; Flags: unchecked

[Files]
; メインプログラム（静的リンク済み）
Source: "target\release\MetaGuard.exe"; DestDir: "{app}"; Flags: ignoreversion sign

; Visual C++ ランタイム（静的リンクのため不要）
; Source: "redist\vc_redist.x64.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall

; 依存DLL（必要に応じて）
; Source: "dlls\*.dll"; DestDir: "{app}"; Flags: ignoreversion

; アイコンファイル（あれば）
; Source: "icon.ico"; DestDir: "{app}"; Flags: ignoreversion

; サンプル設定ファイル
Source: "MetaGuard.ini.sample"; DestDir: "{app}"; DestName: "MetaGuard.ini.sample"; Flags: ignoreversion

; セキュリティ情報ファイル
Source: "SECURITY_INFO.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "INSTALL_GUIDE.txt"; DestDir: "{app}"; Flags: ignoreversion

; ドキュメント
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\設定ファイルを編集"; Filename: "notepad.exe"; Parameters: """{app}\MetaGuard.ini"""
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: quicklaunchicon

[Run]
; Visual C++ ランタイムのインストール（静的リンクのため不要）
; Filename: "{tmp}\vc_redist.x64.exe"; Parameters: "/install /passive /norestart"; StatusMsg: "Visual C++ ランタイムをインストール中..."; Check: VCRedistNeedsInstall

; 初回起動
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Registry]
; 自動起動設定（タスクで選択された場合）
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "MetaGuard"; ValueData: """{app}\{#MyAppExeName}"" --auto"; Flags: uninsdeletevalue; Tasks: autostart

; アプリケーション情報
Root: HKLM; Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; ValueType: string; ValueName: "InstallPath"; ValueData: "{app}"; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; ValueType: string; ValueName: "Version"; ValueData: "{#MyAppVersion}"; Flags: uninsdeletekey

[Code]
// Visual C++ ランタイムのチェック（静的リンクのため不要）
// function VCRedistNeedsInstall: Boolean;
// var
//   Version: String;
// begin
//   Result := False; // 静的リンクのため常にFalse
// end;

// 初回起動時の設定
procedure CurStepChanged(CurStep: TSetupStep);
var
  IniFile: String;
  SampleFile: String;
begin
  if CurStep = ssPostInstall then
  begin
    IniFile := ExpandConstant('{app}\MetaGuard.ini');
    SampleFile := ExpandConstant('{app}\MetaGuard.ini.sample');

    // 設定ファイルが存在しない場合、サンプルからコピー
    if not FileExists(IniFile) and FileExists(SampleFile) then
    begin
      FileCopy(SampleFile, IniFile, False);
    end;
  end;
end;

// アンインストール時の処理
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    // 自動起動設定を削除
    RegDeleteValue(HKEY_CURRENT_USER, 'Software\Microsoft\Windows\CurrentVersion\Run', 'MetaGuard');
  end;
end;