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

    let output_dir_path = PathBuf::from(&output_dir);
    fs::create_dir_all(&output_dir_path)
        .map_err(|err| format!("创建 output_dir 失败: {err}"))?;

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

    // 在 source_dir 中创建本次 run 唯一的临时工作目录
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let temp_dir_name = format!(".krdiffgui_work_{}", timestamp);
    let temp_dir = source_dir_path.join(&temp_dir_name);
    fs::create_dir_all(&temp_dir)
        .map_err(|err| format!("创建临时目录失败: {err}"))?;

    emit_log(&app, format!("source_dir: {source_dir}"));
    emit_log(&app, format!("patch_dir: {patch_dir}"));
    emit_log(&app, format!("output_dir: {output_dir}"));
    emit_log(&app, format!("临时目录: {}", temp_dir.display()));
    emit_log(&app, format!("找到 {} 个补丁文件", patch_files.len()));

    for patch_path in &patch_files {
        let patch_display = patch_path.display().to_string();
        emit_log(&app, format!("正在应用补丁: {patch_display}"));

        // 清空临时目录，准备本次 patch
        clear_dir_contents(&temp_dir)
            .map_err(|err| format!("清空临时目录失败: {err}"))?;

        let mut patcher = KrDiff::new(
            source_dir.clone(),
            patch_path.to_string_lossy().into_owned(),
            temp_dir.to_string_lossy().into_owned(),
        );

        let app_for_progress = app.clone();
        let app_for_log = app.clone();
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
        })),
            Some(Box::new(move |msg| emit_log(&app_for_log, msg))),
        ) {
            Ok(()) => {
                // 将临时目录中的文件复制到 output_dir
                copy_dir_contents(&temp_dir, &output_dir_path)
                    .map_err(|err| format!("复制文件到输出目录失败: {err}"))?;
                emit_log(&app, format!("应用成功: {patch_display}"));
            }
            Err(err) => {
                let _ = fs::remove_dir_all(&temp_dir);
                return Err(format!("{}，补丁文件: {}", err, patch_display));
            }
        }
    }

    // 清理临时目录
    let _ = fs::remove_dir_all(&temp_dir);

    emit_log(&app, "全部补丁应用完成");

    Ok(())
}

/// 递归复制 src 目录下的所有内容到 dst 目录
fn copy_dir_contents(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_contents(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// 清空目录下的所有内容，但保留目录本身
fn clear_dir_contents(dir: &std::path::Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
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
