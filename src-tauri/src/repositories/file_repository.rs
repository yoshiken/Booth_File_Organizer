// FileRepository - ファイル関連操作の責務を分離
// TDDでリファクタリング実施

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use crate::database::{FileRecord, FileUpdateFields};

/// ファイル操作の責務を持つRepository trait
pub trait FileRepository {
    fn insert(&self, file: &FileRecord) -> Result<i64>;
    fn find_by_id(&self, id: i64) -> Result<Option<FileRecord>>;
    fn find_all(&self) -> Result<Vec<FileRecord>>;
    fn delete(&self, file_id: i64) -> Result<Option<String>>;
    fn update_product_fields(&self, file_id: i64, update_fields: &FileUpdateFields) -> Result<()>;
    fn batch_delete(&self, file_ids: &[i64]) -> Result<Vec<String>>;
    fn batch_update(&self, file_ids: &[i64], update_fields: &FileUpdateFields) -> Result<()>;
    
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
                file_path, file_name, file_size, modified_time,
                product_id, product_name, author_name, price,
                description, thumbnail_url, product_url
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                file.file_path,
                file.file_name,
                file.file_size,
                file.modified_time,
                file.product_id,
                file.product_name,
                file.author_name,
                file.price,
                file.description,
                file.thumbnail_url,
                file.product_url,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    fn find_by_id(&self, id: i64) -> Result<Option<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, file_name, file_size, modified_time,
                    created_at, updated_at, product_id, product_name, 
                    author_name, price, description, thumbnail_url, product_url
             FROM files WHERE id = ?1"
        )?;

        let file_iter = stmt.query_map([id], |row| {
            Ok(FileRecord {
                id: Some(row.get(0)?),
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                modified_time: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                product_id: row.get(7)?,
                product_name: row.get(8)?,
                author_name: row.get(9)?,
                price: row.get(10)?,
                description: row.get(11)?,
                thumbnail_url: row.get(12)?,
                product_url: row.get(13)?,
            })
        })?;

        for file in file_iter {
            return Ok(Some(file?));
        }
        Ok(None)
    }

    fn find_all(&self) -> Result<Vec<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, file_name, file_size, modified_time,
                    created_at, updated_at, product_id, product_name,
                    author_name, price, description, thumbnail_url, product_url
             FROM files ORDER BY created_at DESC"
        )?;

        let file_iter = stmt.query_map([], |row| {
            Ok(FileRecord {
                id: Some(row.get(0)?),
                file_path: row.get(1)?,
                file_name: row.get(2)?,
                file_size: row.get(3)?,
                modified_time: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                product_id: row.get(7)?,
                product_name: row.get(8)?,
                author_name: row.get(9)?,
                price: row.get(10)?,
                description: row.get(11)?,
                thumbnail_url: row.get(12)?,
                product_url: row.get(13)?,
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

    fn update_product_fields(&self, file_id: i64, update_fields: &FileUpdateFields) -> Result<()> {
        self.conn.execute(
            "UPDATE files SET 
             product_id = ?1, product_name = ?2, author_name = ?3,
             price = ?4, description = ?5, thumbnail_url = ?6,
             product_url = ?7, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?8",
            params![
                update_fields.product_id,
                update_fields.product_name,
                update_fields.author_name,
                update_fields.price,
                update_fields.description,
                update_fields.thumbnail_url,
                update_fields.product_url,
                file_id
            ],
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
        
        for file_id in file_ids {
            tx.execute(
                "UPDATE files SET 
                 product_id = ?1, product_name = ?2, author_name = ?3,
                 price = ?4, description = ?5, thumbnail_url = ?6,
                 product_url = ?7, updated_at = CURRENT_TIMESTAMP
                 WHERE id = ?8",
                params![
                    update_fields.product_id,
                    update_fields.product_name,
                    update_fields.author_name,
                    update_fields.price,
                    update_fields.description,
                    update_fields.thumbnail_url,
                    update_fields.product_url,
                    file_id
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }


    fn find_all_paginated(&self, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileRecord>, u32)> {
        let sort_by = sort_by.unwrap_or("created_at");
        let sort_order = sort_order.unwrap_or("desc");
        let offset = (page.saturating_sub(1)) * page_size;

        // 検証: sort_byは許可されたカラムのみ
        let allowed_columns = ["id", "file_name", "file_size", "created_at", "updated_at", "author_name", "product_name"];
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
            "SELECT id, file_path, file_name, file_size, modified_time,
                    created_at, updated_at, product_id, product_name,
                    author_name, price, description, thumbnail_url, product_url
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
                modified_time: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                product_id: row.get(7)?,
                product_name: row.get(8)?,
                author_name: row.get(9)?,
                price: row.get(10)?,
                description: row.get(11)?,
                thumbnail_url: row.get(12)?,
                product_url: row.get(13)?,
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
        let allowed_columns = ["id", "file_name", "file_size", "created_at", "updated_at", "author_name", "product_name"];
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
             author_name LIKE ?1 OR 
             product_name LIKE ?1",
            [&search_pattern],
            |row| row.get(0)
        )?;

        // ページネーションされた検索結果を取得
        let search_query = format!(
            "SELECT id, file_path, file_name, file_size, modified_time,
                    created_at, updated_at, product_id, product_name,
                    author_name, price, description, thumbnail_url, product_url
             FROM files 
             WHERE file_name LIKE ?1 OR author_name LIKE ?1 OR product_name LIKE ?1
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
                modified_time: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                product_id: row.get(7)?,
                product_name: row.get(8)?,
                author_name: row.get(9)?,
                price: row.get(10)?,
                description: row.get(11)?,
                thumbnail_url: row.get(12)?,
                product_url: row.get(13)?,
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
                file_path TEXT UNIQUE NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                modified_time INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                product_id TEXT,
                product_name TEXT,
                author_name TEXT,
                price INTEGER,
                description TEXT,
                thumbnail_url TEXT,
                product_url TEXT
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
            file_size: 1024,
            modified_time: 1609459200, // Unix timestamp
            created_at: "2021-01-01T00:00:00Z".to_string(),
            updated_at: "2021-01-01T00:00:00Z".to_string(),
            product_id: Some("12345".to_string()),
            product_name: Some("Test Product".to_string()),
            author_name: Some("Test Shop".to_string()),
            price: Some(500),
            description: Some("Test description".to_string()),
            thumbnail_url: Some("/test/thumb.jpg".to_string()),
            product_url: Some("https://test.booth.pm/items/12345".to_string()),
        };

        // Insert test
        let file_id = repo.insert(&test_file).expect("Failed to insert file");
        assert!(file_id > 0);

        // Find by ID test
        let retrieved_file = repo.find_by_id(file_id).expect("Failed to find file");
        assert!(retrieved_file.is_some());
        
        let file = retrieved_file.unwrap();
        assert_eq!(file.file_name, "file.zip");
        assert_eq!(file.author_name, Some("Test Shop".to_string()));
        assert_eq!(file.file_size, 1024);
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
            file_size: 1024,
            modified_time: 1609459200,
            created_at: "2021-01-01T00:00:00Z".to_string(),
            updated_at: "2021-01-01T00:00:00Z".to_string(),
            product_id: None,
            product_name: None,
            author_name: None,
            price: None,
            description: None,
            thumbnail_url: None,
            product_url: None,
        };

        let file2 = FileRecord {
            id: None,
            file_path: "/test/file2.zip".to_string(),
            file_name: "file2.zip".to_string(),
            file_size: 2048,
            modified_time: 1609459300,
            created_at: "2021-01-01T00:00:00Z".to_string(),
            updated_at: "2021-01-01T00:00:00Z".to_string(),
            product_id: None,
            product_name: None,
            author_name: None,
            price: None,
            description: None,
            thumbnail_url: None,
            product_url: None,
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
            file_size: 1024,
            modified_time: 1609459200,
            created_at: "2021-01-01T00:00:00Z".to_string(),
            updated_at: "2021-01-01T00:00:00Z".to_string(),
            product_id: None,
            product_name: Some("Test Product".to_string()),
            author_name: Some("Test Shop".to_string()),
            price: None,
            description: None,
            thumbnail_url: Some("/test/thumb.jpg".to_string()),
            product_url: None,
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