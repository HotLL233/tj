<#
.SYNOPSIS
    版本源码拷贝脚本 — 精确排除编译产物/依赖包/垃圾文件，确保源码完整性。
.DESCRIPTION
    读取同目录下的 .versionignore，使用 robocopy 高速拷贝，
    并对关键源码文件（.rs / .tsx / .ts）做数量校验，确保不漏。
.PARAMETER Source
    源版本目录路径，例如 D:\桌面\工作量统计工具项目\workload-tool-rust\v0.4.47
.PARAMETER Destination
    目标版本目录路径（会自动创建）
.EXAMPLE
    .\version-copy.ps1 -Source "D:\src\v0.4.47" -Destination "D:\dist\v0.4.48"
#>

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateScript({ Test-Path $_ -PathType Container })]
    [string] $Source,

    [Parameter(Mandatory = $true, Position = 1)]
    [string] $Destination
)

$ErrorActionPreference = "Stop"

# 解析为完整绝对路径
$Source = (Resolve-Path $Source).Path
$Destination = (Resolve-Path $Destination -ErrorAction SilentlyContinue) ?? (
    [System.IO.Path]::GetFullPath($Destination)
)

Write-Host "`n===== version-copy.ps1 =====" -ForegroundColor Cyan
Write-Host "  源目录:     $Source" -ForegroundColor Gray
Write-Host "  目标目录:   $Destination" -ForegroundColor Gray

# ============================================================
# 1. 读取 .versionignore
# ============================================================
$ignoreFile = Join-Path $PSScriptRoot ".versionignore"
if (-not (Test-Path $ignoreFile)) {
    Write-Error "未找到 .versionignore 文件，预期位置: $ignoreFile"
    exit 1
}

$allPatterns = Get-Content $ignoreFile |
    Where-Object { $_ -and $_ -notmatch '^\s*#' } |
    ForEach-Object { $_.Trim() } |
    Where-Object { $_ -ne '' }

Write-Host "`n[1/4] 已加载 $($allPatterns.Count) 条排除规则" -ForegroundColor Yellow

# ============================================================
# 2. 将模式分类为 robocopy 可用的排除列表
# ============================================================
# 目录模式（以 / 结尾）→ robocopy /XD
# 文件模式 → robocopy /XF
# 路径模式（如 frontend/v0.*/）→ 需要仔细处理

$dirExcludes = @()         # robocopy /XD: 只使用目录名（顶层）
$fileExcludes = @()        # robocopy /XF: 文件通配模式
$postCleanupDirs = @()     # 拷贝后需要清理的嵌套路径（相对路径）
$postCleanupFiles = @()    # 拷贝后需要清理的文件（相对路径）
$rootDirPatterns = @()     # 根目录级别的模式

foreach ($p in $allPatterns) {
    if ($p -match '^\*\.(.+)$') {
        # 文件扩展名模式 *.bak *.db 等
        $fileExcludes += "*.$($Matches[1])"
    }
    elseif ($p -match '^v[\d\.]+\*?/$') {
        # 历史版本目录如 v0.*/ — 只匹配根目录
        $rootDirPatterns += $p
    }
    elseif ($p.EndsWith('/')) {
        $dirName = $p.TrimEnd('/')
        # 如果包含 /，说明是嵌套路径（如 frontend/node_modules/）
        if ($dirName -contains '/') {
            $postCleanupDirs += $dirName
        }
        else {
            $dirExcludes += $dirName
        }
    }
    elseif ($p -match '^\*_Setup\.exe$' -or $p -match '^\*_setup\.exe$') {
        $fileExcludes += "*_Setup.exe"
        $fileExcludes += "*_setup.exe"
    }
    elseif ($p -match '^~\$\*') {
        $fileExcludes += "~$*"
    }
    elseif ($p -match '^\*\.(.+)') {
        $fileExcludes += $p
    }
    elseif ($p -match '\.timestamp-.+\.mjs$') {
        $postCleanupFiles += $p
    }
    else {
        # 其他模式（如 $null, nul 等）
        $postCleanupFiles += $p
    }
}

# ============================================================
# 3. 执行 robocopy 主拷贝
# ============================================================
Write-Host "`n[2/4] 开始 robocopy 拷贝..." -ForegroundColor Yellow

if (Test-Path $Destination) {
    Write-Host "  目标目录已存在，将在拷贝前清空" -ForegroundColor DarkYellow
    Remove-Item -Path "$Destination\*" -Recurse -Force -ErrorAction SilentlyContinue
}
else {
    New-Item -Path $Destination -ItemType Directory -Force | Out-Null
}

# 构建 robocopy 参数
$robocopyArgs = @(
    $Source, $Destination,
    "/E",           # 包含子目录（含空目录）
    "/COPY:DAT",    # 复制数据、属性、时间戳
    "/R:2",         # 重试 2 次
    "/W:1",         # 等待 1 秒
    "/NDL",         # 不记录目录列表
    "/NFL",         # 不记录文件列表
    "/NJH",         # 无作业头
    "/NJS",         # 无作业摘要
    "/NP"           # 无进度百分比
)

# 添加目录排除
if ($dirExcludes.Count -gt 0) {
    $robocopyArgs += "/XD"
    $robocopyArgs += $dirExcludes
}

# 添加文件排除
if ($fileExcludes.Count -gt 0) {
    $robocopyArgs += "/XF"
    $robocopyArgs += $fileExcludes
}

Write-Host "  执行: robocopy $($robocopyArgs -join ' ')" -ForegroundColor DarkGray
$robocopyResult = & robocopy @robocopyArgs 2>&1
$exitCode = $LASTEXITCODE

# robocopy 退出码：0-7 正常，8+ 有错误
if ($exitCode -ge 8) {
    Write-Warning "robocopy 返回错误码 $exitCode，尝试继续..."
}

Write-Host "  主拷贝完成 (退出码: $exitCode)" -ForegroundColor Green

# ============================================================
# 4. 后处理：清理嵌套路径
# ============================================================
Write-Host "`n[3/4] 后处理清理..." -ForegroundColor Yellow

# 清理嵌套目录
foreach ($dir in $postCleanupDirs) {
    $targetDir = Join-Path $Destination $dir
    if (Test-Path $targetDir) {
        Remove-Item -Path $targetDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  已清理嵌套目录: $dir" -ForegroundColor DarkGray
    }
}

# 清理特定文件
foreach ($f in $postCleanupFiles) {
    $targetFile = Join-Path $Destination $f
    if (Test-Path $targetFile -PathType Leaf) {
        Remove-Item -Path $targetFile -Force -ErrorAction SilentlyContinue
        Write-Host "  已清理文件: $f" -ForegroundColor DarkGray
    }
}

# 清理根目录级别模式（v0.*/ 等）
foreach ($pattern in $rootDirPatterns) {
    $cleanPattern = $pattern.TrimEnd('/')
    $matches = Get-ChildItem -Path $Destination -Directory -Filter $cleanPattern -ErrorAction SilentlyContinue
    foreach ($m in $matches) {
        Remove-Item -Path $m.FullName -Recurse -Force -ErrorAction SilentlyContinue
        Write-Host "  已清理历史版本目录: $($m.Name)" -ForegroundColor DarkGray
    }
}

# ============================================================
# 5. 校验
# ============================================================
Write-Host "`n[4/4] 校验文件完整性..." -ForegroundColor Yellow

# 统计源目录和目标目录的文件数
$sourceFileCount = (Get-ChildItem -Path $Source -Recurse -File).Count
$destFileCount = (Get-ChildItem -Path $Destination -Recurse -File).Count
$excludedCount = $sourceFileCount - $destFileCount

Write-Host "  源目录文件数:  $sourceFileCount" -ForegroundColor Gray
Write-Host "  目标目录文件数: $destFileCount" -ForegroundColor Gray
Write-Host "  已排除文件数:   $excludedCount" -ForegroundColor Gray

# 对 .rs / .tsx / .ts 做二次校验，确保一个不少
$criticalTypes = @(".rs", ".ts", ".tsx")
$allPass = $true

foreach ($ext in $criticalTypes) {
    $srcCount = (Get-ChildItem -Path $Source -Recurse -File -Filter "*$ext" |
        Where-Object {
            # 排除后端构建产物和嵌套副本中的文件
            $fp = $_.FullName.Replace($Source, "").TrimStart("\")
            $fp -notmatch '^backend\\' -and $fp -notmatch '^target\\' -and
            $fp -notmatch '^dist\\' -and $fp -notmatch '^frontend\\node_modules\\' -and
            $fp -notmatch '^frontend\\src\\src\\'
        }).Count

    $destCount = (Get-ChildItem -Path $Destination -Recurse -File -Filter "*$ext").Count

    if ($srcCount -eq $destCount) {
        Write-Host "  *$ext 文件: 源 $srcCount → 目标 $destCount ✓" -ForegroundColor Green
    }
    else {
        Write-Host "  *$ext 文件: 源 $srcCount → 目标 $destCount ✗ (不匹配!)" -ForegroundColor Red
        $allPass = $false
    }
}

Write-Host "`n===== 拷贝完成 =====" -ForegroundColor Cyan
if ($allPass) {
    Write-Host "状态: ✓ 全部通过，目标位置: $Destination" -ForegroundColor Green
}
else {
    Write-Host "状态: ⚠ 部分校验不通过，请手动检查差异" -ForegroundColor Yellow
}
