; 工作量统计工具 Rust v0.1.1 — Inno Setup 安装脚本

#define MyAppName "工作量统计工具(Rust)"
#define MyAppVersion "0.1.1"
#define MyAppPublisher "HotLL"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{D6E8F0A2-C4B3-4E7F-9D0A-BCDEF2345678}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=D:\桌面\工作量统计工具项目\installer
OutputBaseFilename=工作量统计工具_Rust_v0.1.1_Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#MyAppExeName}
PrivilegesRequired=lowest
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"; GroupDescription: "快捷方式："; Flags: checkedonce

[Files]
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\dist\工作量统计工具_rust_v0.1.1\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "启动 {#MyAppName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\data"
Type: dirifempty; Name: "{app}"
