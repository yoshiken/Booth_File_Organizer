use crate::database::{FileRecord, FileUpdateFields, FileWithTags};
use crate::{AppError, AppState};

// データベース関連のTauriコマンド
#[tauri::command]
pub async fn save_file_to_db(
    state: tauri::State<'_, AppState>,
    file_path: String,
    file_name: String,
    file_size: i64,
    product_url: Option<String>,
    author_name: Option<String>,
    product_name: Option<String>,
    product_id: Option<String>,
    price: Option<i32>,
    description: Option<String>,
    thumbnail_url: Option<String>,
) -> Result<i64, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    let modified_time = std::fs::metadata(&file_path)
        .map(|meta| {
            meta.modified()
                .unwrap_or_else(|_| std::time::SystemTime::now())
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        })
        .unwrap_or_else(|_| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64
        });

    let file_record = FileRecord {
        id: None,
        file_path,
        file_name,
        file_size,
        modified_time,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        product_id,
        product_name,
        author_name,
        price,
        description,
        thumbnail_url,
        product_url,
    };

    db.add_file(file_record).map_err(|e| {
        AppError::file_save(format!("Failed to save file to database: {e}")).to_string()
    })
}

#[tauri::command]
pub async fn get_all_files_from_db(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<FileRecord>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    db.get_all_files().map_err(|e| {
        AppError::file_retrieval(format!("Failed to get files from database: {e}")).to_string()
    })
}

#[tauri::command]
pub async fn get_files_with_tags_from_db(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<FileWithTags>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    db.get_files_with_tags().map_err(|e| {
        AppError::file_retrieval(format!("Failed to get files with tags: {e}")).to_string()
    })
}

#[tauri::command]
pub async fn delete_file_db(
    state: tauri::State<'_, AppState>,
    file_id: i64,
) -> Result<Option<String>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    db.delete_file(file_id).map_err(|e| {
        AppError::file_deletion(format!("Failed to delete file: {e}")).to_string()
    })?;
    Ok(None)
}

#[tauri::command]
pub async fn delete_file_and_folder(
    state: tauri::State<'_, AppState>,
    file_id: i64,
) -> Result<bool, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    // まずファイル情報を取得
    let files = db
        .get_all_files()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files: {e}")).to_string())?;

    let file_info = files.iter().find(|f| f.id == Some(file_id));

    if let Some(file) = file_info {
        let file_path = file.file_path.clone();

        // データベースから削除
        db.delete_file(file_id).map_err(|e| {
            AppError::file_deletion(format!("Failed to delete file from database: {e}"))
                .to_string()
        })?;

        // 物理ファイル・フォルダを削除
        let path = std::path::Path::new(&file_path);

        if path.exists() {
            if path.is_dir() {
                std::fs::remove_dir_all(path).map_err(|e| {
                    AppError::custom(format!("Failed to delete directory: {e}")).to_string()
                })?;
            } else {
                std::fs::remove_file(path).map_err(|e| {
                    AppError::custom(format!("Failed to delete file: {e}")).to_string()
                })?;
            }
        }

        // サムネイルファイルも削除（新しいスキーマでは thumbnail_url フィールド）
        if let Some(thumbnail_url) = &file.thumbnail_url {
            // もしローカルパスの場合は削除を試行
            if thumbnail_url.starts_with("file://") || std::path::Path::new(thumbnail_url).exists()
            {
                let thumb_path = std::path::Path::new(thumbnail_url);
                if thumb_path.exists() {
                    let _ = std::fs::remove_file(thumb_path);
                }
            }
        }

        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn batch_delete_files_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
) -> Result<Vec<String>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    let mut result = Vec::new();

    for file_id in file_ids {
        match db.delete_file(file_id) {
            Ok(_) => {
                // 削除成功時の処理
                result.push(format!("File {file_id} deleted successfully"));
            }
            Err(e) => {
                result.push(format!("Failed to delete file {file_id}: {e}"));
            }
        }
    }

    // バッチ削除後にタグのカウントを再計算
    db.recalculate_usage_counts().map_err(|e| {
        AppError::custom(format!("Failed to recalculate tag usage count: {e}")).to_string()
    })?;

    Ok(result)
}

#[tauri::command]
pub async fn batch_update_files_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
    update_fields: FileUpdateFields,
) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    // バッチ更新の実装（新しいスキーマでは個別更新）
    for file_id in file_ids {
        db.update_file(file_id, update_fields.clone())
            .map_err(|e| {
                AppError::file_update(format!("Failed to update file {file_id}: {e}"))
                    .to_string()
            })?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_files_with_tags_by_ids_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
) -> Result<Vec<FileWithTags>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {e}")).to_string())?;

    // IDでファイルをフィルタリング
    let all_files_with_tags = db.get_files_with_tags().map_err(|e| {
        AppError::file_retrieval(format!("Failed to get files with tags: {e}")).to_string()
    })?;

    let filtered_files = all_files_with_tags
        .into_iter()
        .filter(|file_with_tags| {
            if let Some(id) = file_with_tags.file.id {
                file_ids.contains(&id)
            } else {
                false
            }
        })
        .collect();

    Ok(filtered_files)
}
