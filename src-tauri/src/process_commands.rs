use crate::{AppState, FileSelectResult, ProcessResult, FileRecord, process_zip_internal, AppError};
use log::error;
use std::path::Path;

#[tauri::command]
pub async fn select_zip_files(app: tauri::AppHandle) -> Result<FileSelectResult, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .add_filter("Archive files", &["zip", "rar", "7z", "tar", "gz", "bz2"])
        .set_title("BOOTHアーカイブファイルを選択")
        .pick_files(move |file_paths| {
            if let Err(e) = tx.send(file_paths) {
                error!("Failed to send file paths: {}", e);
            }
        });

    let files = rx
        .recv()
        .map_err(|e| AppError::custom(format!("ファイル選択エラー: {}", e)).to_string())?;

    match files {
        Some(paths) => {
            let file_paths: Vec<String> = paths.iter().map(|p| p.to_string()).collect();

            Ok(FileSelectResult {
                success: true,
                files: file_paths,
                message: format!("{}個のファイルが選択されました", paths.len()),
            })
        }
        None => Ok(FileSelectResult {
            success: false,
            files: vec![],
            message: "ファイルが選択されませんでした".to_string(),
        }),
    }
}

#[tauri::command]
pub async fn process_zip_file(
    state: tauri::State<'_, AppState>,
    zip_path: String,
    booth_url: Option<String>,
    output_dir: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<ProcessResult, String> {
    let booth_client = state.booth_client.clone();
    let result = process_zip_internal(
        zip_path.clone(),
        booth_url.clone(),
        output_dir,
        &booth_client,
    )
    .await;

    match result {
        Ok(mut res) => {
            // データベースに保存を試行
            if res.success {
                let file_name = Path::new(&zip_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let file_size = std::fs::metadata(&zip_path)
                    .map(|meta| meta.len() as i64)
                    .ok();

                let db = state
                    .db
                    .lock()
                    .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;
                let file_record = FileRecord {
                    id: None,
                    file_path: res.output_path.clone().unwrap_or_else(|| zip_path.clone()),
                    file_name,
                    file_size,
                    file_hash: None,
                    booth_product_id: None,
                    booth_shop_name: res.shop_name.clone(),
                    booth_product_name: res.product_name.clone(),
                    booth_url: booth_url.clone(),
                    booth_price: None,
                    booth_thumbnail_path: None,
                    encoding_info: None,
                    created_at: None,
                    updated_at: None,
                    metadata: None,
                };

                match db.insert_file(&file_record) {
                    Ok(file_id) => {
                        // ファイル保存後、タグを追加
                        if let Some(tag_names) = &tags {
                            for tag_name in tag_names {
                                if !tag_name.trim().is_empty() {
                                    if let Ok(tag) = db.get_or_create_tag(tag_name.trim(), None) {
                                        if let Some(tag_id) = tag.id {
                                            let _ = db.add_tag_to_file(file_id, tag_id);
                                        }
                                    }
                                }
                            }
                        }
                        
                        res.message = format!(
                            "{} (ID: {}でデータベースに保存されました)",
                            res.message, file_id
                        );
                    }
                    Err(e) => {
                        error!("Failed to save to database: {}", e);
                        // データベース保存失敗はエラーとしない（ファイル処理は成功しているため）
                    }
                }
            }
            Ok(res)
        }
        Err(e) => Ok(ProcessResult {
            success: false,
            message: format!("処理エラー: {}", e),
            shop_name: None,
            product_name: None,
            files_extracted: vec![],
            output_path: None,
        }),
    }
}

#[tauri::command]
pub async fn select_output_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .set_title("出力先フォルダを選択")
        .pick_folder(move |folder_path| {
            if let Err(e) = tx.send(folder_path) {
                error!("Failed to send folder path: {}", e);
            }
        });

    let folder = rx
        .recv()
        .map_err(|e| AppError::custom(format!("フォルダ選択エラー: {}", e)).to_string())?;

    Ok(folder.map(|p| p.to_string()))
}