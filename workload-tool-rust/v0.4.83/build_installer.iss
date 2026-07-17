#define MyAppVersion "0.4.83"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName=鏍峰搧绠＄悊绯荤粺
AppVersion={#MyAppVersion}
VersionInfoProductName=鏍峰搧绠＄悊绯荤粺
VersionInfoProductVersion={#MyAppVersion}
VersionInfoVersion={#MyAppVersion}
UninstallDisplayName=鏍峰搧绠＄悊绯荤粺
AppPublisher=WorkloadTool
DefaultDirName={autopf}\鏍峰搧绠＄悊绯荤粺
OutputDir=D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.83\installer
OutputBaseFilename=样品管理系统_v0.4.83_Setup
SetupIconFile=D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.83\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "鍒涘缓妗岄潰蹇嵎鏂瑰紡"

[InstallDelete]
Type: filesandordirs; Name: "{app}\static"

[Files]
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.83\dist\workload-tool.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.83\backend\static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs
Source: "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.83\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\鏍峰搧绠＄悊绯荤粺"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\icon.ico"
Name: "{group}\鍗歌浇 鏍峰搧绠＄悊绯荤粺"; Filename: "{uninstallexe}"
Name: "{autodesktop}\鏍峰搧绠＄悊绯荤粺"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\icon.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\static"
Type: dirifempty; Name: "{app}"

