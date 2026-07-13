# v0.4.39 版本更新说明

## 更新日期
2026-07-12

## 更新内容
### 修复：导出功能异常（真正根因）
- **根因**：Axios 的 `responseType: 'blob'` 存在已知 bug——后端返回 500 等非 2xx 错误时，Axios 走 Promise reject 链（.catch），`.then()` 里的 `r.status !== 200` 分支永远执行不到，导致 JSON 错误信息被当作 Blob 传到 catch 然后吞掉，用户点"导出 Excel"完全没反应。
- **修复**：三个导出函数（exportExcel / exportRdExcel / exportSampleInfo）的 `responseType: 'blob'` 全部改为 `responseType: 'arraybuffer'`。
  - 成功时：手动 `new Blob([arraybuffer])` 创建下载文件
  - 失败时：`new TextDecoder().decode(arraybuffer)` 正确解析后端返回的 JSON 错误
- **改动文件**：仅 `frontend/src/api/client.ts`

### 补充修复：错误提示可视化
- StatsPage 主页面添加全局 `<Alert>` 错误提示

## 打包说明
- 无 cmd 窗口（生产模式 `cargo build --release`）
- 支持覆盖安装（AppId 固定）
