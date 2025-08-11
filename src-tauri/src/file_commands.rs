use crate::{AppState, FileRecord, FileWithTags, FileUpdateFields, AppError};

// データベース関連のTauriコマンド
#[tauri::command]
pub async fn save_file_to_db(
    state: tauri::State<'_, AppState>,
    file_path: String,
    file_name: String,
    file_size: Option<i64>,
    booth_url: Option<String>,
    shop_name: Option<String>,
    product_name: Option<String>,
) -> Result<i64, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    let file_record = FileRecord {
        id: None,
        file_path,
        file_name,
        file_size,
        file_hash: None,
        booth_product_id: None,
        booth_shop_name: shop_name,
        booth_product_name: product_name,
        booth_url,
        booth_price: None,
        booth_thumbnail_path: None,
        encoding_info: None,
        created_at: None,
        updated_at: None,
        metadata: None,
    };

    db.insert_file(&file_record)
        .map_err(|e| AppError::file_save(format!("Failed to save file to database: {}", e)).to_string())
}

#[tauri::command]
pub async fn get_all_files_from_db(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<FileRecord>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.get_all_files()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files from database: {}", e)).to_string())
}

#[tauri::command]
pub async fn get_files_with_tags_from_db(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<FileWithTags>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.get_files_with_tags()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files with tags: {}", e)).to_string())
}

#[tauri::command]
pub async fn delete_file_db(
    state: tauri::State<'_, AppState>,
    file_id: i64,
) -> Result<Option<String>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.delete_file(file_id)
        .map_err(|e| AppError::file_deletion(format!("Failed to delete file: {}", e)).to_string())
}

#[tauri::command]
pub async fn delete_file_and_folder(
    state: tauri::State<'_, AppState>,
    file_id: i64,
) -> Result<bool, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    // データベースからファイル情報を取得して削除
    if let Some(file_path) = db.delete_file(file_id)
        .map_err(|e| AppError::file_deletion(format!("Failed to delete file from database: {}", e)).to_string())? {
        
        // 物理ファイル・フォルダを削除
        let path = std::path::Path::new(&file_path);
        
        if path.exists() {
            if path.is_dir() {
                std::fs::remove_dir_all(path)
                    .map_err(|e| AppError::custom(format!("Failed to delete directory: {}", e)).to_string())?;
            } else {
                std::fs::remove_file(path)
                    .map_err(|e| AppError::custom(format!("Failed to delete file: {}", e)).to_string())?;
            }
        }
        
        // サムネイルファイルも削除
        if let Ok(Some(file_info)) = db.get_file_by_id(file_id) {
            if let Some(thumbnail_path) = file_info.booth_thumbnail_path {
                let thumb_path = std::path::Path::new(&thumbnail_path);
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
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    let result = db.batch_delete_files(&file_ids)
        .map_err(|e| AppError::file_deletion(format!("Failed to batch delete files: {}", e)).to_string())?;
    
    // バッチ削除後にタグのカウントを再計算
    db.recalculate_tag_usage_count()
        .map_err(|e| AppError::custom(format!("Failed to recalculate tag usage count: {}", e)).to_string())?;
    
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
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.batch_update_files(&file_ids, &update_fields)
        .map_err(|e| AppError::file_update(format!("Failed to batch update files: {}", e)).to_string())
}

#[tauri::command]
pub async fn get_files_with_tags_by_ids_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
) -> Result<Vec<FileWithTags>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.get_files_with_tags_by_ids(&file_ids)
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files with tags by IDs: {}", e)).to_string())
}