//! 二维码工具：将内容生成为 PNG 并写入数据目录，返回（绝对路径, data URL）。
//!
//! 说明：qrcode 0.12 内部依赖 image 0.23，而本项目 tray 等模块使用 image 0.24 API，
//! 二者 `Luma<u8>` 类型不互通。因此这里用 qrcode 解析模块矩阵，再用 image 0.24 手动绘制灰度 PNG，
//! 避免 qrcode 的 image 渲染特性带来的版本耦合。

use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use image::{GrayImage, Luma};
use qrcode::QrCode;

use crate::error::Result;

/// 解析数据目录（与 AppConfig::data_dir 保持一致）：优先 WORKLOAD_DATA_DIR，否则 exe 同级 data 目录。
fn data_dir() -> std::path::PathBuf {
    if let Ok(d) = std::env::var("WORKLOAD_DATA_DIR") {
        return std::path::PathBuf::from(d);
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default()
        .join("data")
}

/// 生成二维码 PNG。
/// - 写入 `{data_dir}/qr/{filename}`
/// - 返回 (绝对路径, data:image/png;base64,... 可直接用于 <img src>)
pub fn generate(content: &str, filename: &str) -> Result<(String, String)> {
    let dir = data_dir().join("qr");
    std::fs::create_dir_all(&dir)?;
    let abs_path = dir.join(filename);

    let qr = QrCode::new(content)
        .map_err(|e| crate::error::AppError::Validation(format!("二维码内容无效: {}", e)))?;
    let scale: u32 = 8; // 每个模块像素大小
    let count = qr.width() as u32;
    let dim = count * scale;
    let mut img: GrayImage = GrayImage::new(dim, dim);
    for y in 0..count {
        for x in 0..count {
            // qrcode 的 Index<(usize,usize)> 返回该模块是否为深色
            let v = qr[(x as usize, y as usize)].select(0u8, 255u8);
            for dy in 0..scale {
                for dx in 0..scale {
                    img.put_pixel(x * scale + dx, y * scale + dy, Luma([v]));
                }
            }
        }
    }
    img.save(&abs_path)?;

    // 回读 PNG 字节以生成 data URL
    let png_bytes = std::fs::read(&abs_path)?;
    let data_url = format!("data:image/png;base64,{}", BASE64_STANDARD.encode(&png_bytes));
    Ok((abs_path.to_string_lossy().to_string(), data_url))
}
