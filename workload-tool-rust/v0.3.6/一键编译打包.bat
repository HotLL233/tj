@echo off
chcp 65001 >nul
echo ========================================
echo   工作量统计工具 v0.3.6 一键编译打包
echo ========================================
echo.

cd /d "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.5"

REM 步骤 1: 编译
echo [步骤 1/2] 编译 Release 版本...
echo.
call compile.bat
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo 编译失败，停止打包。
    pause
    exit /b 1
)

echo.
echo ========================================
echo.

REM 步骤 2: 打包
echo [步骤 2/2] 执行打包脚本...
echo.
powershell -ExecutionPolicy Bypass -File "build-and-pack.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo 打包失败。
    pause
    exit /b 1
)

echo.
echo ========================================
echo   全部完成！
echo ========================================
echo.
echo 安装包位置:
echo   D:\桌面\工作量统计工具项目\workload-tool-rust\安装包\v0.3.6
echo.
pause
