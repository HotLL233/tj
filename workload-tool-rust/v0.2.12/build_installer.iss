#define MyAppName "工作量统计工具"
#define MyAppVersion "0.2.12"
#define MyAppPublisher "WorkloadTool"
#define MyAppURL "http://localhost:8000"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=D:\桌面\工作量统计工具项目\installer
OutputBaseFilename=工作量统计工具_Rust_v0.2.12_Setup
SetupIconFile=D:\桌面\工作量统计工具项目\workload-tool-rust\v0.2.12\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"

[Files]
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.2.12\dist\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.2.12\static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.2.12\icon.ico"; DestDir: "{app}"; Flags: ignoreversion
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
