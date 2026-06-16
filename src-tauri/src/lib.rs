// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
use std::fs;
use std::path::PathBuf;
mod utils;
pub mod patchers;

use patchers::KrDiff;


#[tauri::command]
fn apply_krdiff_dir(source_dir: String, patch_dir: String, output_dir: String) -> Result<Vec<String>, String> {
    let mut logs = Vec::new();

    if source_dir.trim().is_empty() {
        return Err("source_dir 不能为空".to_string());
    }

    if patch_dir.trim().is_empty() {
        return Err("patch_dir 不能为空".to_string());
    }

    if output_dir.trim().is_empty() {
        return Err("output_dir 不能为空".to_string());
    }

    let patch_dir_path = PathBuf::from(&patch_dir);

    if !patch_dir_path.exists() {
        return Err(format!("patch_dir 不存在: {}", patch_dir_path.display()));
    }

    if !patch_dir_path.is_dir() {
        return Err(format!("patch_dir 不是目录: {}", patch_dir_path.display()));
    }

    let mut patch_files: Vec<PathBuf> = fs::read_dir(&patch_dir_path)
        .map_err(|err| format!("读取 patch 目录失败: {err}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("krpdiff") || ext.eq_ignore_ascii_case("krdiff")
                })
        })
        .collect();

    patch_files.sort();

    if patch_files.is_empty() {
        return Err(format!(
            "没有找到 .krpdiff 或 .krdiff 文件: {}",
            patch_dir_path.display()
        ));
    }

    logs.push(format!("source_dir: {source_dir}"));
    logs.push(format!("patch_dir: {patch_dir}"));
    logs.push(format!("output_dir: {output_dir}"));
    logs.push(format!("找到 {} 个补丁文件", patch_files.len()));

    for patch_path in patch_files {
        logs.push(format!("正在应用补丁: {}", patch_path.display()));

        let mut patcher = KrDiff::new(
            source_dir.clone(),
            patch_path.to_string_lossy().into_owned(),
            output_dir.clone(),
        );

        if patcher.apply() {
            logs.push(format!("应用成功: {}", patch_path.display()));
        } else {
            return Err(format!("KrDiff patch failed: {}", patch_path.display()));
        }
    }

    logs.push("全部补丁应用完成".to_string());

    Ok(logs)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![apply_krdiff_dir])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}