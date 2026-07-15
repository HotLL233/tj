#define MyAppVersion "0.4.72"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName=样品管理系统
AppVersion={#MyAppVersion}
AppPublisher=WorkloadTool
DefaultDirName={autopf}\样品管理系统
OutputDir=D:\桌面\工作量统计工具项目\installer
OutputBaseFilename=样品管理系统_v0.4.72_Setup
SetupIconFile=D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.72\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"

[Files]
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.72\dist\workload-tool.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.72\backend\static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.72\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\样品管理系统"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 样品管理系统"; Filename: "{uninstallexe}"
Name: "{autodesktop}\样品管理系统"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\data"
Type: filesandordirs; Name: "{app}\static"
Type: dirifempty; Name: "{app}"
