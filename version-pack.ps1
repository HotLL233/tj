<#
.SYNOPSIS
    版本打包脚本 — 执行版本拷贝 → 构建后端 → 构建前端 → InnoSetup 打包。
.DESCRIPTION
    以 InnoSetup 打包为蓝本，完整复现正式安装包构建流程：
    1. 调用 version-copy.ps1 将源码拷贝到临时构建目录
    2. 在临时目录中执行 cargo build --release（Rust 后端）
    3. 在临时目录中执行 npm install && npm run build（前端）
    4. 将编译产物放到 .iss 预期的位置
    5. 调用 iscc 编译 .iss 脚本生成安装包
.PARAMETER Source
    源版本目录路径（含完整 Rust 源码 + frontend 前端 + build_installer.iss）
.PARAMETER Config
    打包配置：debug 或 release（默认 release）
.PARAMETER OutputDir
    安装包输出目录（默认使用 .iss 中定义的 OutputDir）
.EXAMPLE
    .\version-pack.ps1 -Source "D:\src\workload-tool-rust\v0.4.47"
#>

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateScript({ Test-Path $_ -PathType Container })]
    [string] $Source,

    [ValidateSet("debug", "release")]
    [string] $Config = "release",

    [string] $OutputDir = ""
)

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# 解析源目录
$Source = (Resolve-Path $Source).Path
Write-Host "`n===== version-pack.ps1 =====" -ForegroundColor Magenta
Write-Host "  源码目录:         $Source" -ForegroundColor Gray
Write-Host "  构建配置:         $Config" -ForegroundColor Gray

# ============================================================
# 0. 前置检查：确保必要工具可用
# ============================================================
Write-Host "`n[0/6] 前置检查..." -ForegroundColor Yellow

# 检查 Rust/Cargo
$cargoPath = Get-Command "cargo" -ErrorAction SilentlyContinue
if (-not $cargoPath) {
    Write-Error "未找到 cargo，请确保已安装 Rust 工具链"
    exit 1
}
Write-Host "  cargo: $($cargoPath.Source)" -ForegroundColor Gray

# 检查 Node.js/npm
$npmPath = Get-Command "npm" -ErrorAction SilentlyContinue
if (-not $npmPath) {
    Write-Error "未找到 npm，请确保已安装 Node.js"
    exit 1
}
Write-Host "  npm:   $($npmPath.Source)" -ForegroundColor Gray

# 检查 InnoSetup
$isccPath = Get-Command "iscc" -ErrorAction SilentlyContinue
if (-not $isccPath) {
    # 尝试常见安装路径
    $possiblePaths = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\iscc.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 5\iscc.exe",
        "${env:ProgramFiles}\Inno Setup 6\iscc.exe",
        "${env:ProgramFiles}\Inno Setup 5\iscc.exe"
    )
    foreach ($p in $possiblePaths) {
        if (Test-Path $p) {
            $isccPath = $p
            break
        }
    }
    if (-not $isccPath) {
        Write-Warning "未找到 iscc（InnoSetup 编译器），请确保已安装 Inno Setup"
        Write-Warning "安装后请将 iscc.exe 所在目录加入 PATH 环境变量"
        Write-Warning "继续执行，但最后一步打包将失败"
    }
    else {
        Write-Host "  iscc:  $isccPath" -ForegroundColor Gray
    }
}
else {
    Write-Host "  iscc:  $isccPath.Source" -ForegroundColor Gray
}

# ============================================================
# 1. 创建临时构建目录
# ============================================================
Write-Host "`n[1/6] 创建临时构建目录..." -ForegroundColor Yellow

$version = if ($Source -match 'v(\d+\.\d+\.\d+)') { $matches[1] } else { "temp" }
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$stagingDir = Join-Path $env:TEMP "workload-pack-$version-$stamp"
New-Item -Path $stagingDir -ItemType Directory -Force | Out-Null
Write-Host "  临时目录: $stagingDir" -ForegroundColor Gray

# ============================================================
# 2. 版本拷贝（调用 version-copy.ps1）
# ============================================================
Write-Host "`n[2/6] 版本源码拷贝..." -ForegroundColor Yellow

$copyScript = Join-Path $scriptDir "version-copy.ps1"
if (-not (Test-Path $copyScript)) {
    Write-Error "未找到 version-copy.ps1，预期位置: $copyScript"
    exit 1
}

& $copyScript -Source $Source -Destination $stagingDir
if ($LASTEXITCODE -ne 0) {
    Write-Error "版本拷贝失败，退出码: $LASTEXITCODE"
    exit 1
}

# ============================================================
# 3. 构建 Rust 后端
# ============================================================
Write-Host "`n[3/6] 构建 Rust 后端 (cargo build --$Config)..." -ForegroundColor Yellow
Write-Host "  这可能需要 3-15 分钟，请耐心等待..." -ForegroundColor DarkYellow

Push-Location $stagingDir
try {
    $buildResult = & cargo build --$Config 2>&1
    $buildExitCode = $LASTEXITCODE
    if ($buildExitCode -ne 0) {
        Write-Error "Rust 构建失败 (退出码: $buildExitCode)"
        Write-Host "最后 30 行输出:" -ForegroundColor Red
        $buildResult | Select-Object -Last 30 | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
        exit 1
    }
    Write-Host "  Rust 构建成功" -ForegroundColor Green
}
finally {
    Pop-Location
}

# 确定编译产物路径
$exeName = "workload-tool.exe"
$releaseExe = Join-Path $stagingDir "target\$Config\$exeName"
if (-not (Test-Path $releaseExe)) {
    Write-Error "编译产物未找到: $releaseExe"
    exit 1
}

# ============================================================
# 4. 构建前端
# ============================================================
Write-Host "`n[4/6] 构建前端 (npm install && npm run build)..." -ForegroundColor Yellow

$frontendDir = Join-Path $stagingDir "frontend"
if (-not (Test-Path $frontendDir)) {
    Write-Warning "未找到 frontend 目录，跳过前端构建"
}
else {
    Push-Location $frontendDir
    try {
        Write-Host "  npm install..." -ForegroundColor Gray
        $npmInstall = & npm install 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "npm install 失败"
            exit 1
        }
        Write-Host "  npm install 完成" -ForegroundColor Gray

        Write-Host "  npm run build..." -ForegroundColor Gray
        $npmBuild = & npm run build 2>&1
        if ($LASTEXITCODE -ne 0) {
            Write-Error "npm run build 失败"
            Write-Host "最后 20 行输出:" -ForegroundColor Red
            $npmBuild | Select-Object -Last 20 | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
            exit 1
        }
        Write-Host "  前端构建成功" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}

# ============================================================
# 5. 整理产物：将 exe 放到 dist/ 位置（.iss 脚本期望的位置）
# ============================================================
Write-Host "`n[5/6] 整理产物..." -ForegroundColor Yellow

# 创建 dist/ 目录并拷贝 exe
$distDir = Join-Path $stagingDir "dist"
if (-not (Test-Path $distDir)) {
    New-Item -Path $distDir -ItemType Directory -Force | Out-Null
}
Copy-Item -Path $releaseExe -Destination (Join-Path $distDir $exeName) -Force
Write-Host "  编译产物 → dist\$exeName" -ForegroundColor Gray

# 确认 backend/static/ 已存在（vite 应该已输出到此处）
$backendStatic = Join-Path $stagingDir "backend\static"
if (Test-Path $backendStatic) {
    $staticCount = (Get-ChildItem -Path $backendStatic -Recurse -File).Count
    Write-Host "  前端产物 → backend\static\ ($staticCount 个文件)" -ForegroundColor Gray
}
else {
    Write-Warning "backend\static\ 不存在，前端构建可能未输出到预期位置"
    Write-Warning "vite.config.ts 中 build.outDir 应为 '../backend/static'"
}

# ============================================================
# 6. 执行 InnoSetup 打包
# ============================================================
Write-Host "`n[6/6] InnoSetup 打包..." -ForegroundColor Yellow

# 查找构建目录中的 .iss 脚本
$issScript = Join-Path $stagingDir "build_installer.iss"
if (-not (Test-Path $issScript)) {
    Write-Error "未找到 build_installer.iss，预期位置: $issScript"
    exit 1
}

# 如果需要自定义输出目录，临时修改 .iss
if ($OutputDir) {
    if (-not (Test-Path $OutputDir)) {
        New-Item -Path $OutputDir -ItemType Directory -Force | Out-Null
    }
    $issContent = Get-Content $issScript -Raw
    $issContent = $issContent -replace 'OutputDir=.*', "OutputDir=$OutputDir"
    Set-Content -Path $issScript -Value $issContent
    Write-Host "  已设置输出目录: $OutputDir" -ForegroundColor Gray
}

Push-Location $stagingDir
try {
    if ($isccPath -is [string]) {
        # 通过完整路径调用
        $isccResult = & $isccPath $issScript 2>&1
    }
    else {
        $isccResult = & iscc $issScript 2>&1
    }
    $isccExitCode = $LASTEXITCODE

    if ($isccExitCode -ne 0) {
        Write-Error "InnoSetup 打包失败 (退出码: $isccExitCode)"
        $isccResult | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
        exit 1
    }

    # 查找生成的安装包
    $setupExe = Get-ChildItem -Path $stagingDir -Recurse -Filter "*_Setup.exe" | Select-Object -First 1
    if (-not $setupExe) {
        # 可能在 OutputDir 中
        $outputDir = if ($OutputDir) { $OutputDir } else { Join-Path $stagingDir "installer" }
        $setupExe = Get-ChildItem -Path $outputDir -Recurse -Filter "*_Setup.exe" | Select-Object -First 1
    }

    Write-Host "`n===== 打包完成 =====" -ForegroundColor Magenta
    if ($setupExe) {
        Write-Host "安装包: $($setupExe.FullName)" -ForegroundColor Green
        Write-Host "大小:    $('{0:N1}' -f ($setupExe.Length / 1MB)) MB" -ForegroundColor Gray

        # 输出到项目根目录的 installer/
        $targetInstaller = "D:\桌面\工作量统计工具项目\installer"
        if (-not (Test-Path $targetInstaller)) {
            New-Item -Path $targetInstaller -ItemType Directory -Force | Out-Null
        }
        Copy-Item -Path $setupExe.FullName -Destination $targetInstaller -Force
        Write-Host "已复制到: $targetInstaller" -ForegroundColor Green
    }
    else {
        Write-Host "InnoSetup 编译完成，但未找到生成的安装包文件" -ForegroundColor Yellow
        Write-Host "请检查 .iss 中的 OutputDir 配置" -ForegroundColor Yellow
    }
}
finally {
    Pop-Location
}

# 清理临时构建目录
Write-Host "`n清理临时目录..." -ForegroundColor DarkGray
Remove-Item -Path $stagingDir -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "  已删除: $stagingDir" -ForegroundColor DarkGray

Write-Host "`n===== version-pack.ps1 执行完毕 =====" -ForegroundColor Magenta
