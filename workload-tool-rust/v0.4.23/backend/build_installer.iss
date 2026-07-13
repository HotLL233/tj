#define MyAppVersion "0.4.23"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{A69C2C8C-1D53-4DD3-B737-1DF7B9CF3638}
AppName=本地化LIMS
AppVersion={#MyAppVersion}
AppPublisher=本地化LIMS
DefaultDirName={autopf}\本地化LIMS
OutputDir=D:\桌面\工作量统计工具项目\installer
OutputBaseFilename=本地化LIMS-0.4.23-setup
SetupIconFile=icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "chinesesimp"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"

[Files]
; 源路径相对本脚本所在目录（v0.4.23/backend）。
; 打包前需先：cargo build --release （生成 target\release\workload-tool.exe）
;             npm run build        （生成 static\，前端产物）
Source: "target\release\workload-tool.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "static\*"; DestDir: "{app}\static"; Flags: ignoreversion recursesubdirs
Source: "icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\本地化LIMS"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 本地化LIMS"; Filename: "{uninstallexe}"
Name: "{autodesktop}\本地化LIMS"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; 注意：不再删除 {app}\data，升级/重装时保留用户数据（SQLite 库）。
Type: filesandordirs; Name: "{app}\static"
Type: dirifempty; Name: "{app}"
