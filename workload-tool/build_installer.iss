; 工作量统计工具 v1.0.0 Rust版 安装程序

#define MyAppName "工作量统计工具"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "HotLL"
#define MyAppURL "https://github.com/HotLL233/hot"
#define MyAppExeName "workload-tool.exe"

[Setup]
AppId={{C3D5E7F9-B1A2-4D6E-8C9F-ABCDEF123456}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={localappdata}\工作量统计工具
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=D:\桌面\workload-tool\installer
OutputBaseFilename=工作量统计工具_v1.0.0_Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#MyAppExeName}
PrivilegesRequired=lowest
SetupIconFile=D:\桌面\py\app_icon.ico

[Languages]
Name: "chinesesimplified"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[Tasks]
Name: "desktopicon"; Description: "创建桌面快捷方式"; GroupDescription: "快捷方式："; Flags: checkedonce

[Files]
Source: "D:\桌面\workload-tool\src-tauri\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\卸载 {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "运行 工作量统计工具"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\{#MyAppName}\logs"
Type: dirifempty; Name: "{localappdata}\{#MyAppName}"
