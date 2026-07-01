; 工作量统计工具 (Rust) v0.2.2 — Inno Setup 安装脚本

#define MyAppName "工作量统计工具 (Rust)"
#define MyAppVersion "0.2.2"
#define MyAppPublisher "HotLL"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{D6E8F0A2-C4B3-4E7F-9D0A-BCDEF2345679}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=D:\桌面\工作量统计工具项目\installer
OutputBaseFilename=工作量统计工具_Rust_v0.2.2_Setup
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
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.2.2\dist\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.2.2\static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.2.2\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\excel-parser\target\release\excel-parser.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "启动 {#MyAppName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: files; Name: "{app}\excel-parser.exe"
Type: filesandordirs; Name: "{app}\data"
Type: filesandordirs; Name: "{app}\static"
Type: dirifempty; Name: "{app}"
