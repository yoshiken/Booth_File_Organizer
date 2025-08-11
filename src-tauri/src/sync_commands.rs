use crate::{AppState, SyncResult, MissingFile, AppError};
use crate::database::{FileWithTags, BatchStatistics};

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

    // 新しいスキーマでは簡単な検索を実装（ファイル名でフィルタリング）
    let all_files_with_tags = db.get_files_with_tags()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files with tags: {}", e)).to_string())?;
    
    let filtered_files = all_files_with_tags.into_iter()
        .filter(|file_with_tags| {
            let file_name = &file_with_tags.file.file_name;
            let file_path = &file_with_tags.file.file_path;
            let product_name = file_with_tags.file.product_name.as_deref().unwrap_or("");
            let author_name = file_with_tags.file.author_name.as_deref().unwrap_or("");
            
            file_name.to_lowercase().contains(&query.to_lowercase()) ||
            file_path.to_lowercase().contains(&query.to_lowercase()) ||
            product_name.to_lowercase().contains(&query.to_lowercase()) ||
            author_name.to_lowercase().contains(&query.to_lowercase())
        })
        .collect();
    
    Ok(filtered_files)
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

    
    // タグでフィルタリングする新しい実装
    let all_files_with_tags = db.get_files_with_tags()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files with tags: {}", e)).to_string())?;
    
    let filtered_files = all_files_with_tags.into_iter()
        .filter(|file_with_tags| {
            // ファイルが指定されたタグのいずれかを持っているかチェック
            let file_tag_names: Vec<String> = file_with_tags.tags.iter()
                .map(|tag| tag.name.clone())
                .collect();
                
            tagNames.iter().any(|tag_name| {
                file_tag_names.iter().any(|file_tag| file_tag == tag_name)
            })
        })
        .collect();
    
    Ok(filtered_files)
}

#[tauri::command]
pub async fn find_duplicate_files_db(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Vec<FileWithTags>>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| AppError::database_lock(format!("Database lock error: {}", e)).to_string())?;

    // 重複ファイル検出の新しい実装（ファイルサイズと名前で比較）
    let all_files_with_tags = db.get_files_with_tags()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get files with tags: {}", e)).to_string())?;
    
    let mut duplicate_groups: Vec<Vec<FileWithTags>> = Vec::new();
    let mut processed_files: std::collections::HashSet<i64> = std::collections::HashSet::new();
    
    for (i, file1) in all_files_with_tags.iter().enumerate() {
        if let Some(id1) = file1.file.id {
            if processed_files.contains(&id1) {
                continue;
            }
            
            let mut duplicate_group = vec![file1.clone()];
            processed_files.insert(id1);
            
            for file2 in all_files_with_tags.iter().skip(i + 1) {
                if let Some(id2) = file2.file.id {
                    if processed_files.contains(&id2) {
                        continue;
                    }
                    
                    // 同じファイルサイズとファイル名で重複と判定
                    if file1.file.file_size == file2.file.file_size && 
                       file1.file.file_name == file2.file.file_name {
                        duplicate_group.push(file2.clone());
                        processed_files.insert(id2);
                    }
                }
            }
            
            if duplicate_group.len() > 1 {
                duplicate_groups.push(duplicate_group);
            }
        }
    }
    
    Ok(duplicate_groups)
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

    // バッチ統計の新しい実装
    let all_files = db.get_all_files()
        .map_err(|e| AppError::file_retrieval(format!("Failed to get all files: {}", e)).to_string())?;
    
    let selected_files = all_files.into_iter()
        .filter(|file| file.id.map_or(false, |id| file_ids.contains(&id)))
        .collect::<Vec<_>>();
    
    let processed = selected_files.len();
    let updated = selected_files.iter()
        .filter(|file| file.product_name.is_some() || file.author_name.is_some())
        .count();
    
    Ok(BatchStatistics {
        processed,
        updated,
        errors: 0,
    })
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
                booth_shop_name: file_record.author_name.clone(), // author_name を使用
                booth_product_name: file_record.product_name.clone(),
            });
        }
    }

    // ファイル同期後にタグのカウントを再計算
    db.recalculate_usage_counts()
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
        match db.delete_file(file_id) {
            Ok(_) => {
                removed_count += 1;
            }
            Err(e) => {
                // Log error but continue with other files
                eprintln!("Failed to delete file {}: {}", file_id, e);
            }
        }
    }

    // ファイル削除後にタグのカウントを再計算
    db.recalculate_usage_counts()
        .map_err(|e| AppError::custom(format!("Failed to recalculate tag usage count: {}", e)).to_string())?;

    Ok(removed_count)
}