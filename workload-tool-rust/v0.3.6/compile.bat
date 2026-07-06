@echo off
chcp 65001 >nul
echo ========================================
echo   工作量统计工具 v0.3.6 编译打包
echo ========================================
echo.
echo 正在启动编译...
echo.

cd /d "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.5"

echo [1/3] 清理旧的编译产物...
if exist "target\release\workload-tool.exe" (
    del /f /q "target\release\workload-tool.exe"
    echo   已清理旧文件
)
echo.

echo [2/3] 开始编译 Release 版本（预计 30-60 秒）...
echo   执行命令: cargo build --release
echo.
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [错误] 编译失败！错误代码: %ERRORLEVEL%
    echo.
    pause
    exit /b 1
)

echo.
echo [3/3] 验证编译产物...
if exist "target\release\workload-tool.exe" (
    echo   [√] workload-tool.exe 已生成
    for %%A in ("target\release\workload-tool.exe") do (
        set size=%%~zA
        set /a sizeMB=!size! / 1048576
        echo   [√] 文件大小: !sizeMB! MB
    )
) else (
    echo   [×] 未找到可执行文件
    pause
    exit /b 1
)

echo.
echo ========================================
echo   编译完成！
echo ========================================
echo.
echo 可执行文件位置:
echo   target\release\workload-tool.exe
echo.
echo 下一步: 运行打包脚本
echo   命令: powershell -ExecutionPolicy Bypass -File build-and-pack.ps1
echo.
pause
