// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tauri::Emitter;

pub mod patchers;
mod utils;

use patchers::KrDiff;

fn emit_log(app: &tauri::AppHandle, message: impl Into<String>) {
    let _ = app.emit("patch-log", message.into());
}

#[tauri::command]
async fn apply_krdiff_dir(
    app: tauri::AppHandle,
    source_dir: String,
    patch_dir: String,
    output_dir: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        apply_krdiff_dir_blocking(app, source_dir, patch_dir, output_dir)
    })
    .await
    .map_err(|err| format!("补丁任务执行失败: {err}"))?
}

fn apply_krdiff_dir_blocking(
    app: tauri::AppHandle,
    source_dir: String,
    patch_dir: String,
    output_dir: String,
) -> Result<(), String> {
    if source_dir.trim().is_empty() {
        return Err("source_dir 不能为空".to_string());
    }

    if patch_dir.trim().is_empty() {
        return Err("patch_dir 不能为空".to_string());
    }

    if output_dir.trim().is_empty() {
        return Err("output_dir 不能为空".to_string());
    }

    let source_dir_path = PathBuf::from(&source_dir);
    if !source_dir_path.exists() {
        return Err(format!("source_dir 不存在: {}", source_dir_path.display()));
    }
    if !source_dir_path.is_dir() {
        return Err(format!(
            "source_dir 不是目录: {}",
            source_dir_path.display()
        ));
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

    emit_log(&app, format!("source_dir: {source_dir}"));
    emit_log(&app, format!("patch_dir: {patch_dir}"));
    emit_log(&app, format!("output_dir: {output_dir}"));
    emit_log(&app, format!("找到 {} 个补丁文件", patch_files.len()));

    for patch_path in patch_files {
        let patch_display = patch_path.display().to_string();
        emit_log(&app, format!("正在应用补丁: {patch_display}"));

        let mut patcher = KrDiff::new(
            source_dir.clone(),
            patch_path.to_string_lossy().into_owned(),
            output_dir.clone(),
        );

        let app_for_progress = app.clone();
        let patch_for_progress = patch_display.clone();
        let mut last_emit = Instant::now();

        match patcher.apply_with_progress(Some(Box::new(move |written_bytes| {
            if last_emit.elapsed() >= Duration::from_millis(5000) {
                let mib = written_bytes as f64 / 1024.0 / 1024.0;
                let _ = app_for_progress.emit(
                    "patch-log",
                    format!("正在合并: {patch_for_progress}，已写入 {mib:.2} MiB"),
                );
                last_emit = Instant::now();
            }
        }))) {
            Ok(()) => {
                emit_log(&app, format!("应用成功: {patch_display}"));
            }
            Err(err) => {
                return Err(format!("{}，补丁文件: {}", err, patch_display));
            }
        }
    }

    emit_log(&app, "全部补丁应用完成");

    Ok(())
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
