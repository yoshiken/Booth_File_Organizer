use crate::{AppState, FileWithTags, BatchStatistics, SyncResult, MissingFile, AppError};

// Phase 3: 検索・重複検出コマンド
#[tauri::command]
pub async fn search_files_db(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<FileWithTags>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    // 検索結果を取得
    let files = db.search_files(&query)
        .map_err(|e| AppError::file_retrieval(format!("Failed to search files: {}", e)).to_string())?;
    
    // ファイルIDを抽出
    let file_ids: Vec<i64> = files.iter()
        .filter_map(|f| f.file.id.map(|id| id as i64))
        .collect();
    
    // ファイルIDからタグ付きファイル情報を取得
    if file_ids.is_empty() {
        return Ok(vec![]);
    }
    
    db.get_files_with_tags_by_ids(&file_ids)
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files with tags: {}", e)).to_string())
}

#[tauri::command]
pub async fn search_files_by_tags_db(
    state: tauri::State<'_, AppState>,
    tagNames: Vec<String>,
) -> Result<Vec<FileWithTags>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    
    // タグで検索してファイルを取得
    let files = db.search_files_by_tags(&tagNames)
        .map_err(|e| AppError::file_retrieval(format!("Failed to search files by tags: {}", e)).to_string())?;
    
    // ファイルIDを抽出
    let file_ids: Vec<i64> = files.iter()
        .filter_map(|f| f.file.id.map(|id| id as i64))
        .collect();
    
    // ファイルIDからタグ付きファイル情報を取得
    if file_ids.is_empty() {
        return Ok(vec![]);
    }
    
    let result = db.get_files_with_tags_by_ids(&file_ids)
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files with tags: {}", e)).to_string())?;
    Ok(result)
}

#[tauri::command]
pub async fn find_duplicate_files_db(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Vec<FileWithTags>>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.find_duplicate_files()
        .map_err(|e| AppError::file_retrieval(format!("Failed to find duplicate files: {}", e)).to_string())
}

#[tauri::command]
pub async fn get_batch_statistics_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
) -> Result<BatchStatistics, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    db.get_batch_statistics(&file_ids)
        .map_err(|e| AppError::file_retrieval(format!("Failed to get batch statistics: {}", e)).to_string())
}

#[tauri::command]
pub async fn sync_file_system_db(
    state: tauri::State<'_, AppState>,
) -> Result<SyncResult, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    // データベースから全ファイルを取得
    let all_files = db.get_all_files()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get all files: {}", e)).to_string())?;

    let mut sync_result = SyncResult {
        total_files: all_files.len(),
        missing_files: Vec::new(),
        orphaned_files: 0,
        updated_files: 0,
    };

    for file_record in all_files {
        // ファイルの存在確認
        if !std::path::Path::new(&file_record.file_path).exists() {
            sync_result.missing_files.push(MissingFile {
                id: file_record.id.unwrap_or(0),
                file_name: file_record.file_name.clone(),
                file_path: file_record.file_path.clone(),
                booth_shop_name: file_record.booth_shop_name.clone(),
                booth_product_name: file_record.booth_product_name.clone(),
            });
        }
    }

    // ファイル同期後にタグのカウントを再計算
    db.recalculate_tag_usage_count()
        .map_err(|e| AppError::custom(format!("Failed to recalculate tag usage count: {}", e)).to_string())?;

    Ok(sync_result)
}

#[tauri::command]
pub async fn remove_missing_files_db(
    state: tauri::State<'_, AppState>,
    file_ids: Vec<i64>,
) -> Result<usize, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    let mut removed_count = 0;
    for file_id in file_ids {
        if db.delete_file(file_id).map_err(|e| AppError::file_deletion(format!("Failed to delete file: {}", e)).to_string())?.is_some() {
            removed_count += 1;
        }
    }

    // ファイル削除後にタグのカウントを再計算
    db.recalculate_tag_usage_count()
        .map_err(|e| AppError::custom(format!("Failed to recalculate tag usage count: {}", e)).to_string())?;

    Ok(removed_count)
}