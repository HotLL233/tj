# v0.3.6 自动编译打包脚本
# 功能：编译 Release 版本并打包到安装包目录

$ErrorActionPreference = "Stop"

$version = "0.3.6"
$projectRoot = "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.5"
$packageRoot = "D:\桌面\工作量统计工具项目\workload-tool-rust\安装包"
$packageDir = "$packageRoot\v$version"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  工作量统计工具 v$version 编译打包" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 步骤 1: 检查项目目录
Write-Host "[1/5] 检查项目目录..." -ForegroundColor Yellow
if (-not (Test-Path $projectRoot)) {
    Write-Host "  [×] 项目目录不存在: $projectRoot" -ForegroundColor Red
    exit 1
}
Set-Location $projectRoot
Write-Host "  [√] 项目目录: $projectRoot" -ForegroundColor Green

# 步骤 2: 编译项目
Write-Host "`n[2/5] 开始编译 Release 版本..." -ForegroundColor Yellow
Write-Host "  编译模式: Release (无控制台)" -ForegroundColor Gray
Write-Host "  预计时间: 30-60 秒" -ForegroundColor Gray

$buildStart = Get-Date
try {
    cargo build --release 2>&1 | Out-String | Write-Host
    $buildEnd = Get-Date
    $buildTime = ($buildEnd - $buildStart).TotalSeconds
    Write-Host "  [√] 编译完成 (耗时: $([math]::Round($buildTime, 1))秒)" -ForegroundColor Green
} catch {
    Write-Host "  [×] 编译失败" -ForegroundColor Red
    Write-Host $_.Exception.Message -ForegroundColor Red
    exit 1
}

# 步骤 3: 验证编译产物
Write-Host "`n[3/5] 验证编译产物..." -ForegroundColor Yellow
$exePath = "$projectRoot\target\release\workload-tool.exe"
if (-not (Test-Path $exePath)) {
    Write-Host "  [×] 可执行文件不存在: $exePath" -ForegroundColor Red
    exit 1
}
$exeSize = (Get-Item $exePath).Length / 1MB
Write-Host "  [√] workload-tool.exe ($([math]::Round($exeSize, 2)) MB)" -ForegroundColor Green

# 步骤 4: 创建打包目录
Write-Host "`n[4/5] 准备打包目录..." -ForegroundColor Yellow
if (Test-Path $packageDir) {
    Write-Host "  清理旧版本..." -ForegroundColor Gray
    Remove-Item $packageDir -Recurse -Force
}
New-Item -ItemType Directory -Force -Path $packageDir | Out-Null
Write-Host "  [√] 打包目录: $packageDir" -ForegroundColor Green

# 步骤 5: 复制文件
Write-Host "`n[5/5] 复制文件..." -ForegroundColor Yellow

# 可执行文件
Copy-Item $exePath -Destination $packageDir -Force
Write-Host "  [√] workload-tool.exe" -ForegroundColor Green

# 静态资源
if (Test-Path "$projectRoot\static") {
    Copy-Item "$projectRoot\static" -Destination "$packageDir\static" -Recurse -Force
    $staticCount = (Get-ChildItem "$packageDir\static" -Recurse -File).Count
    Write-Host "  [√] static\ ($staticCount 个文件)" -ForegroundColor Green
} else {
    Write-Host "  [!] 未找到 static 目录" -ForegroundColor Yellow
}

# 配置文件示例
if (Test-Path "$projectRoot\config.toml") {
    Copy-Item "$projectRoot\config.toml" -Destination "$packageDir\config.toml.example" -Force
    Write-Host "  [√] config.toml.example" -ForegroundColor Green
}

# 图标
if (Test-Path "$projectRoot\icon.ico") {
    Copy-Item "$projectRoot\icon.ico" -Destination $packageDir -Force
    Write-Host "  [√] icon.ico" -ForegroundColor Green
}

# README
if (Test-Path "$projectRoot\README.md") {
    Copy-Item "$projectRoot\README.md" -Destination $packageDir -Force
    Write-Host "  [√] README.md" -ForegroundColor Green
}

# 编译指南
if (Test-Path "$projectRoot\编译打包指南_v0.3.6.md") {
    Copy-Item "$projectRoot\编译打包指南_v0.3.6.md" -Destination $packageDir -Force
    Write-Host "  [√] 编译打包指南_v0.3.6.md" -ForegroundColor Green
}

# 创建启动脚本
$startScript = @"
@echo off
echo ========================================
echo   工作量统计工具 v$version
echo ========================================
echo.
echo 正在启动服务器...
start workload-tool.exe
echo.
echo 服务器已启动！
echo 请在浏览器中访问: http://localhost:8000
echo.
echo 按任意键退出...
pause >nul
"@
$startScript | Out-File -FilePath "$packageDir\启动.bat" -Encoding ASCII -Force
Write-Host "  [√] 启动.bat" -ForegroundColor Green

# 创建版本说明
$versionInfo = @"
# 工作量统计工具 v$version

## 版本信息
- 版本号: $version
- 编译日期: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")
- 编译耗时: $([math]::Round($buildTime, 1))秒
- 程序大小: $([math]::Round($exeSize, 2)) MB

## 主要更新
- 导出模块完全重构
- 新增 10 个 Sheet 的 Excel 导出
- 优化数据查询性能
- 增强公式计算准确性

## 快速启动
1. 双击 "启动.bat" 启动服务器
2. 浏览器访问 http://localhost:8000
3. 首次运行会自动创建 data 目录和数据库

## 配置说明
如需修改配置，请复制 config.toml.example 为 config.toml，
并根据需要修改以下配置项：
- server_port: 服务器端口（默认 8000）
- db_dir: 数据库目录（默认 data）
- log_level: 日志级别（默认 info）

## 技术支持
- 查看日志: app.log
- 数据库: data/workload.db
- 备份: data/backups/

---
编译者: Claude AI
"@
$versionInfo | Out-File -FilePath "$packageDir\版本说明.txt" -Encoding UTF8 -Force
Write-Host "  [√] 版本说明.txt" -ForegroundColor Green

# 完成
Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "  打包完成！" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "`n安装包位置:" -ForegroundColor Yellow
Write-Host "  $packageDir" -ForegroundColor Cyan
Write-Host "`n文件列表:" -ForegroundColor Yellow
Get-ChildItem $packageDir | Format-Table Name, Length, LastWriteTime -AutoSize

$totalSize = (Get-ChildItem $packageDir -Recurse -File | Measure-Object -Property Length -Sum).Sum / 1MB
Write-Host "`n总大小: $([math]::Round($totalSize, 2)) MB" -ForegroundColor Cyan

Write-Host "`n按任意键退出..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
