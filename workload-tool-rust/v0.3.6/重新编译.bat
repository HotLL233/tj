@echo off
chcp 65001 >nul
echo ========================================
echo   v0.3.6 导出错误修复 - 重新编译
echo ========================================
echo.

cd /d "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.6"

echo [步骤 1/3] 清理旧的编译产物...
cargo clean
echo   清理完成
echo.

echo [步骤 2/3] 重新编译 Release 版本...
echo   预计时间: 30-60 秒
echo.
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [错误] 编译失败！
    pause
    exit /b 1
)

echo.
echo [步骤 3/3] 验证编译产物...
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
echo 下一步: 测试导出功能
echo   1. 双击运行: test-export.bat
echo   或
echo   2. 手动运行: cargo run --features console
echo      然后在浏览器中测试导出
echo.
pause
