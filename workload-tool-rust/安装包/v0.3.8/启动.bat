@echo off
chcp 65001 >nul
cls
echo ========================================
echo   工作量统计工具 v0.3.8
echo ========================================
echo.
echo 正在启动服务器...
start "" workload-tool.exe
timeout /t 2 >nul
echo.
echo [成功] 服务器已启动！
echo.
echo 请在浏览器中访问: http://localhost:8000
echo.
echo 提示：
echo  - 关闭此窗口不会停止服务器
echo  - 退出服务器请在系统托盘图标右键点击"退出"
echo  - 数据存储在 data 目录
echo  - 日志文件: app.log
echo.
pause
