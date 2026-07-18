#define MyAppVersion "0.4.95.0"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName=工作量统计工具
AppVersion={#MyAppVersion}
AppPublisher=WorkloadTool
DefaultDirName={autopf}\工作量统计工具
OutputDir=installer
OutputBaseFilename=工作量统计工具_Rust_v0.4.95_布局改造预览版_Setup
SetupIconFile=icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"

[Files]
Source: "dist\workload-tool.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "backend\static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\工作量统计工具"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 工作量统计工具"; Filename: "{uninstallexe}"
Name: "{autodesktop}\工作量统计工具"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\static"
Type: dirifempty; Name: "{app}"
