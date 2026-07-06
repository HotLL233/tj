@echo off
chcp 65001 >nul
echo ========================================
echo   测试导出功能（带控制台日志）
echo ========================================
echo.

cd /d "D:\桌面\工作量统计工具项目\workload-tool-rust\v0.3.6"

echo 正在启动服务器（带控制台）...
echo.
echo 请观察日志输出：
echo   - Sheet 1 查询完成: X 行
echo   - Sheet 1 写入完成
echo   - ... (依次到 Sheet 10)
echo   - Excel 生成成功: XXXXX bytes
echo.
echo 启动后请：
echo   1. 在浏览器打开 http://localhost:8000
echo   2. 测试导出 Excel
echo   3. 检查文件大小（应该 50KB+）
echo   4. 打开 Excel 检查 10 个 Sheet
echo.
echo 按 Ctrl+C 停止服务器
echo.
echo ========================================
echo.

cargo run --features console
