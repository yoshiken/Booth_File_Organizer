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

