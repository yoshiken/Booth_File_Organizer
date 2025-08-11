// FileRepository - ファイル関連操作の責務を分離
// TDDでリファクタリング実施

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use crate::database_refactored::{FileRecord, FileUpdateFields, BatchStatistics};

/// ファイル操作の責務を持つRepository trait
pub trait FileRepository {
    fn insert(&self, file: &FileRecord) -> Result<i64>;
    fn find_by_id(&self, id: i64) -> Result<Option<FileRecord>>;
    fn find_all(&self) -> Result<Vec<FileRecord>>;
    fn delete(&self, file_id: i64) -> Result<Option<String>>;
    fn update_booth_url(&self, file_id: i64, booth_url: Option<&str>) -> Result<()>;
    fn batch_delete(&self, file_ids: &[i64]) -> Result<Vec<String>>;
    fn batch_update(&self, file_ids: &[i64], update_fields: &FileUpdateFields) -> Result<()>;
    fn get_batch_statistics(&self, file_ids: &[i64]) -> Result<BatchStatistics>;
    
    // 新しいページネーションメソッド
    fn find_all_paginated(&self, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileRecord>, u32)>;
    fn search_paginated(&self, query: &str, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileRecord>, u32)>;
}

/// SQLite実装のFileRepository
pub struct SqliteFileRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteFileRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> FileRepository for SqliteFileRepository<'a> {
    fn insert(&self, file: &FileRecord) -> Result<i64> {
        let _id = self.conn.execute(
            "INSERT INTO files (
                file_path, file_name, file_size, file_hash, booth_product_id,
                booth_shop_name, booth_product_name, booth_url, booth_price,
                booth_thumbnail_path, encoding_info, metadata, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, datetime('now'))",
            params![
                file.file_path,
                file.file_name,
                file.file_size,
                file.file_hash,
                file.booth_product_id,
                file.booth_shop_name,
                file.booth_product_name,
                file.booth_url,
                file.booth_price,
                file.booth_thumbnail_path,
                file.encoding_info,
                file.metadata,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    fn find_by_id(&self, id: i64) -> Result<Option<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, file_name, file_size, file_hash, booth_product_id,
                    booth_shop_name, booth_product_name, booth_url, booth_price,
                    booth_thumbnail_path, encoding_info, created_at, updated_at, metadata
             FROM files WHERE id = ?1"
        )?;

        let file_iter = stmt.query_map([id], |row| {
            Ok(FileRecord {
                id: Some(row.get(0)?),
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                file_hash: row.get(4)?,
                booth_product_id: row.get(5)?,
                booth_shop_name: row.get(6)?,
                booth_product_name: row.get(7)?,
                booth_url: row.get(8)?,
                booth_price: row.get(9)?,
                booth_thumbnail_path: row.get(10)?,
                encoding_info: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                metadata: row.get(14)?,
            })
        })?;

        for file in file_iter {
            return Ok(Some(file?));
        }
        Ok(None)
    }

    fn find_all(&self) -> Result<Vec<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, file_name, file_size, file_hash, booth_product_id,
                    booth_shop_name, booth_product_name, booth_url, booth_price,
                    booth_thumbnail_path, encoding_info, created_at, updated_at, metadata
             FROM files ORDER BY created_at DESC"
        )?;

        let file_iter = stmt.query_map([], |row| {
            Ok(FileRecord {
                id: Some(row.get(0)?),
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                file_hash: row.get(4)?,
                booth_product_id: row.get(5)?,
                booth_shop_name: row.get(6)?,
                booth_product_name: row.get(7)?,
                booth_url: row.get(8)?,
                booth_price: row.get(9)?,
                booth_thumbnail_path: row.get(10)?,
                encoding_info: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                metadata: row.get(14)?,
            })
        })?;

        let mut files = Vec::new();
        for file in file_iter {
            files.push(file?);
        }
        Ok(files)
    }



    fn delete(&self, file_id: i64) -> Result<Option<String>> {
        // ファイルパスを取得
        let file_path: Option<String> = self.conn.query_row(
            "SELECT file_path FROM files WHERE id = ?1",
            [file_id],
            |row| row.get(0)
        ).optional()?;

        if let Some(_path) = &file_path {
            // ファイルレコードを削除（タグ関連は外部キー制約でカスケード削除）
            self.conn.execute("DELETE FROM files WHERE id = ?1", [file_id])?;
        }

        Ok(file_path)
    }

    fn update_booth_url(&self, file_id: i64, booth_url: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE files SET booth_url = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![booth_url, file_id],
        )?;
        Ok(())
    }

    fn batch_delete(&self, file_ids: &[i64]) -> Result<Vec<String>> {
        let mut deleted_paths = Vec::new();
        let tx = self.conn.unchecked_transaction()?;
        
        for file_id in file_ids {
            // ファイルパスを取得
            let path_result = tx.query_row(
                "SELECT file_path FROM files WHERE id = ?1",
                [file_id],
                |row| row.get::<_, String>(0)
            ).optional()?;
            
            if let Some(path) = path_result {
                deleted_paths.push(path);
                // ファイルレコードを削除
                tx.execute("DELETE FROM files WHERE id = ?1", [file_id])?;
            }
        }
        
        tx.commit()?;
        Ok(deleted_paths)
    }

    fn batch_update(&self, file_ids: &[i64], update_fields: &FileUpdateFields) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let mut update_clauses = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

        if let Some(shop_name) = &update_fields.booth_shop_name {
            update_clauses.push("booth_shop_name = ?");
            params.push(shop_name);
        }
        if let Some(product_name) = &update_fields.booth_product_name {
            update_clauses.push("booth_product_name = ?");
            params.push(product_name);
        }
        if let Some(url) = &update_fields.booth_url {
            update_clauses.push("booth_url = ?");
            params.push(url);
        }
        if let Some(metadata) = &update_fields.metadata {
            update_clauses.push("metadata = ?");
            params.push(metadata);
        }

        update_clauses.push("updated_at = datetime('now')");

        let update_sql = format!(
            "UPDATE files SET {} WHERE id = ?",
            update_clauses.join(", ")
        );

        for file_id in file_ids {
            params.push(file_id);
            tx.execute(&update_sql, &params[..])?;
            params.pop(); // Remove file_id for next iteration
        }

        tx.commit()?;
        Ok(())
    }

    fn get_batch_statistics(&self, file_ids: &[i64]) -> Result<BatchStatistics> {
        if file_ids.is_empty() {
            return Ok(BatchStatistics {
                total_files: 0,
                total_size: None,
                unique_shops: 0,
                unique_products: 0,
            });
        }

        let placeholders: Vec<String> = file_ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT 
                COUNT(*) as total_files,
                SUM(file_size) as total_size,
                COUNT(DISTINCT booth_shop_name) as unique_shops,
                COUNT(DISTINCT booth_product_id) as unique_products
             FROM files 
             WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut stmt = self.conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> = file_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        
        let stats = stmt.query_row(&params[..], |row| {
            Ok(BatchStatistics {
                total_files: row.get(0)?,
                total_size: row.get(1)?,
                unique_shops: row.get(2)?,
                unique_products: row.get(3)?,
            })
        })?;

        Ok(stats)
    }

    fn find_all_paginated(&self, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileRecord>, u32)> {
        let sort_by = sort_by.unwrap_or("created_at");
        let sort_order = sort_order.unwrap_or("desc");
        let offset = (page.saturating_sub(1)) * page_size;

        // 検証: sort_byは許可されたカラムのみ
        let allowed_columns = ["id", "file_name", "file_size", "created_at", "updated_at", "booth_shop_name", "booth_product_name"];
        if !allowed_columns.contains(&sort_by) {
            return Err(anyhow::anyhow!("Invalid sort_by column: {}", sort_by));
        }

        // 検証: sort_orderは "asc" または "desc" のみ
        let sort_order = match sort_order.to_lowercase().as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            _ => return Err(anyhow::anyhow!("Invalid sort_order: {}", sort_order)),
        };

        // 総数を取得
        let total_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM files",
            [],
            |row| row.get(0)
        )?;

        // ページネーションされたファイルを取得
        let query = format!(
            "SELECT id, file_path, file_name, file_size, file_hash, booth_product_id,
                    booth_shop_name, booth_product_name, booth_url, booth_price,
                    booth_thumbnail_path, encoding_info, created_at, updated_at, metadata
             FROM files ORDER BY {} {} LIMIT ? OFFSET ?",
            sort_by, sort_order
        );

        let mut stmt = self.conn.prepare(&query)?;
        let file_iter = stmt.query_map([page_size, offset], |row| {
            Ok(FileRecord {
                id: Some(row.get(0)?),
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                file_hash: row.get(4)?,
                booth_product_id: row.get(5)?,
                booth_shop_name: row.get(6)?,
                booth_product_name: row.get(7)?,
                booth_url: row.get(8)?,
                booth_price: row.get(9)?,
                booth_thumbnail_path: row.get(10)?,
                encoding_info: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                metadata: row.get(14)?,
            })
        })?;

        let mut files = Vec::new();
        for file in file_iter {
            files.push(file?);
        }

        Ok((files, total_count))
    }

    fn search_paginated(&self, query: &str, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileRecord>, u32)> {
        let sort_by = sort_by.unwrap_or("created_at");
        let sort_order = sort_order.unwrap_or("desc");
        let offset = (page.saturating_sub(1)) * page_size;

        // 検証: sort_byは許可されたカラムのみ
        let allowed_columns = ["id", "file_name", "file_size", "created_at", "updated_at", "booth_shop_name", "booth_product_name"];
        if !allowed_columns.contains(&sort_by) {
            return Err(anyhow::anyhow!("Invalid sort_by column: {}", sort_by));
        }

        // 検証: sort_orderは "asc" または "desc" のみ
        let sort_order = match sort_order.to_lowercase().as_str() {
            "asc" => "ASC",
            "desc" => "DESC",
            _ => return Err(anyhow::anyhow!("Invalid sort_order: {}", sort_order)),
        };

        let search_pattern = format!("%{}%", query);

        // 検索結果の総数を取得
        let total_count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM files WHERE 
             file_name LIKE ?1 OR 
             booth_shop_name LIKE ?1 OR 
             booth_product_name LIKE ?1",
            [&search_pattern],
            |row| row.get(0)
        )?;

        // ページネーションされた検索結果を取得
        let search_query = format!(
            "SELECT id, file_path, file_name, file_size, file_hash, booth_product_id,
                    booth_shop_name, booth_product_name, booth_url, booth_price,
                    booth_thumbnail_path, encoding_info, created_at, updated_at, metadata
             FROM files 
             WHERE file_name LIKE ?1 OR booth_shop_name LIKE ?1 OR booth_product_name LIKE ?1
             ORDER BY {} {} LIMIT ? OFFSET ?",
            sort_by, sort_order
        );

        let mut stmt = self.conn.prepare(&search_query)?;
        let file_iter = stmt.query_map([&search_pattern, &page_size.to_string(), &offset.to_string()], |row| {
            Ok(FileRecord {
                id: Some(row.get(0)?),
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                file_hash: row.get(4)?,
                booth_product_id: row.get(5)?,
                booth_shop_name: row.get(6)?,
                booth_product_name: row.get(7)?,
                booth_url: row.get(8)?,
                booth_price: row.get(9)?,
                booth_thumbnail_path: row.get(10)?,
                encoding_info: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
                metadata: row.get(14)?,
            })
        })?;

        let mut files = Vec::new();
        for file in file_iter {
            files.push(file?);
        }

        Ok((files, total_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use tempfile::NamedTempFile;

    fn create_test_db() -> (NamedTempFile, Connection) {
        let db_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(db_file.path()).unwrap();
        
        // テーブル作成
        conn.execute(
            "CREATE TABLE files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER,
                file_hash TEXT,
                booth_product_id INTEGER,
                booth_shop_name TEXT,
                booth_product_name TEXT,
                booth_url TEXT,
                booth_price INTEGER,
                booth_thumbnail_path TEXT,
                encoding_info TEXT,
                created_at TEXT,
                updated_at TEXT,
                metadata TEXT
            )",
            [],
        ).unwrap();

        (db_file, conn)
    }

    #[test]
    fn test_insert_and_find_by_id() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteFileRepository::new(&conn);

        let test_file = FileRecord {
            id: None,
            file_path: "/test/file.zip".to_string(),
            file_name: "file.zip".to_string(),
            file_size: Some(1024),
            file_hash: Some("test_hash".to_string()),
            booth_product_id: Some(12345),
            booth_shop_name: Some("Test Shop".to_string()),
            booth_product_name: Some("Test Product".to_string()),
            booth_url: Some("https://test.booth.pm/items/12345".to_string()),
            booth_price: Some(500),
            booth_thumbnail_path: Some("/test/thumb.jpg".to_string()),
            encoding_info: Some("UTF-8".to_string()),
            created_at: None,
            updated_at: None,
            metadata: Some("test metadata".to_string()),
        };

        // Insert test
        let file_id = repo.insert(&test_file).expect("Failed to insert file");
        assert!(file_id > 0);

        // Find by ID test
        let retrieved_file = repo.find_by_id(file_id).expect("Failed to find file");
        assert!(retrieved_file.is_some());
        
        let file = retrieved_file.unwrap();
        assert_eq!(file.file_name, "file.zip");
        assert_eq!(file.booth_shop_name, Some("Test Shop".to_string()));
        assert_eq!(file.file_size, Some(1024));
    }

    #[test]
    fn test_find_all() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteFileRepository::new(&conn);

        // Initially empty
        let files = repo.find_all().expect("Failed to get all files");
        assert_eq!(files.len(), 0);

        // Insert two files
        let file1 = FileRecord {
            id: None,
            file_path: "/test/file1.zip".to_string(),
            file_name: "file1.zip".to_string(),
            file_size: Some(1024),
            file_hash: Some("hash1".to_string()),
            booth_product_id: None,
            booth_shop_name: None,
            booth_product_name: None,
            booth_url: None,
            booth_price: None,
            booth_thumbnail_path: None,
            encoding_info: None,
            created_at: None,
            updated_at: None,
            metadata: None,
        };

        let file2 = FileRecord {
            id: None,
            file_path: "/test/file2.zip".to_string(),
            file_name: "file2.zip".to_string(),
            file_size: Some(2048),
            file_hash: Some("hash2".to_string()),
            booth_product_id: None,
            booth_shop_name: None,
            booth_product_name: None,
            booth_url: None,
            booth_price: None,
            booth_thumbnail_path: None,
            encoding_info: None,
            created_at: None,
            updated_at: None,
            metadata: None,
        };

        repo.insert(&file1).expect("Failed to insert file1");
        repo.insert(&file2).expect("Failed to insert file2");

        // Should find both files
        let files = repo.find_all().expect("Failed to get all files");
        assert_eq!(files.len(), 2);
    }


    #[test]
    fn test_delete() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteFileRepository::new(&conn);

        let test_file = FileRecord {
            id: None,
            file_path: "/test/delete_me.zip".to_string(),
            file_name: "delete_me.zip".to_string(),
            file_size: Some(1024),
            file_hash: Some("delete_hash".to_string()),
            booth_product_id: None,
            booth_shop_name: Some("Test Shop".to_string()),
            booth_product_name: Some("Test Product".to_string()),
            booth_url: None,
            booth_price: None,
            booth_thumbnail_path: Some("/test/thumb.jpg".to_string()),
            encoding_info: None,
            created_at: None,
            updated_at: None,
            metadata: None,
        };

        let file_id = repo.insert(&test_file).expect("Failed to insert file");

        // Verify file exists
        assert!(repo.find_by_id(file_id).expect("Failed to get file").is_some());

        // Delete the file
        let deleted_path = repo.delete(file_id).expect("Failed to delete file");
        assert_eq!(deleted_path, Some("/test/delete_me.zip".to_string()));

        // Verify file is deleted
        assert!(repo.find_by_id(file_id).expect("Failed to get file").is_none());

        // Test deleting non-existent file
        let result = repo.delete(999999).expect("Failed to handle non-existent file");
        assert_eq!(result, None);
    }

}