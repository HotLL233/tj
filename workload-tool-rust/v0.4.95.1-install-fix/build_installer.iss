#define MyAppVersion "0.4.95.1"
#define MyAppNumericVersion "0.4.95.1"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName=样品管理系统
AppVersion={#MyAppVersion}
VersionInfoProductName=样品管理系统
VersionInfoProductVersion={#MyAppNumericVersion}
VersionInfoVersion={#MyAppNumericVersion}
UninstallDisplayName=样品管理系统
AppPublisher=WorkloadTool
DefaultDirName={autopf}\样品管理系统
UsePreviousAppDir=yes
OutputDir=installer
OutputBaseFilename=样品管理系统_v0.4.95.1_安装覆盖修复版_Setup
SetupIconFile=icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"

[InstallDelete]
Type: filesandordirs; Name: "{app}\static"

[Files]
Source: "dist\workload-tool.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "backend\static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\样品管理系统"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\icon.ico"
Name: "{group}\卸载 样品管理系统"; Filename: "{uninstallexe}"
Name: "{autodesktop}\样品管理系统"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\icon.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\static"
Type: dirifempty; Name: "{app}"
