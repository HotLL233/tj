#define MyAppVersion "0.3.15"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName=工作量统计工具
AppVersion={#MyAppVersion}
AppPublisher=WorkloadTool
DefaultDirName={autopf}\工作量统计工具
OutputDir=D:\桌面\工作量统计工具项目\installer
OutputBaseFilename=工作量统计工具_Rust_v0.3.15_Setup
SetupIconFile=D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.15\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"

[Files]
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.15\dist\workload-tool.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.15\static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.15\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\工作量统计工具"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 工作量统计工具"; Filename: "{uninstallexe}"
Name: "{autodesktop}\工作量统计工具"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\data"
Type: filesandordirs; Name: "{app}\static"
Type: dirifempty; Name: "{app}"
