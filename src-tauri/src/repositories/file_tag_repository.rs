// FileTagRepository - ファイル-タグ関連操作の責務を分離
// TDDでリファクタリング実施

use anyhow::Result;
use rusqlite::{params, Connection};
use crate::database_refactored::{FileRecord, FileWithTags, Tag};

/// ファイル-タグ関連操作の責務を持つRepository trait
pub trait FileTagRepository {
    fn add_tag_to_file(&self, file_id: i64, tag_id: i64) -> Result<()>;
    fn remove_tag_from_file(&self, file_id: i64, tag_id: i64) -> Result<()>;
    fn remove_tag_from_file_by_name(&self, file_id: i64, tag_name: &str) -> Result<()>;
    fn get_tags_for_file(&self, file_id: i64) -> Result<Vec<Tag>>;
    fn get_files_with_tags(&self) -> Result<Vec<FileWithTags>>;
    fn get_files_with_tags_by_ids(&self, file_ids: &[i64]) -> Result<Vec<FileWithTags>>;
    fn batch_add_tag_to_files(&self, file_ids: &[i64], tag_id: i64) -> Result<()>;
    fn batch_remove_tag_from_files(&self, file_ids: &[i64], tag_id: i64) -> Result<()>;
    
    // 新しいページネーションメソッド
    fn get_files_with_tags_paginated(&self, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileWithTags>, u32)>;
    fn search_files_with_tags_by_tags_paginated(&self, tag_names: &[String], page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileWithTags>, u32)>;
}

/// SQLite実装のFileTagRepository
pub struct SqliteFileTagRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteFileTagRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> FileTagRepository for SqliteFileTagRepository<'a> {
    fn add_tag_to_file(&self, file_id: i64, tag_id: i64) -> Result<()> {
        // ファイル-タグ関連を追加
        self.conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id, added_at)
             VALUES (?1, ?2, datetime('now'))",
            params![file_id, tag_id],
        )?;

        // タグの使用回数を増加
        self.conn.execute(
            "UPDATE tags SET usage_count = usage_count + 1 WHERE id = ?1",
            params![tag_id],
        )?;

        Ok(())
    }

    fn remove_tag_from_file(&self, file_id: i64, tag_id: i64) -> Result<()> {
        // ファイル-タグ関連を削除
        let affected_rows = self.conn.execute(
            "DELETE FROM file_tags WHERE file_id = ?1 AND tag_id = ?2",
            params![file_id, tag_id],
        )?;

        // 実際に削除された場合のみタグの使用回数を減少
        if affected_rows > 0 {
            self.conn.execute(
                "UPDATE tags SET usage_count = MAX(0, usage_count - 1) WHERE id = ?1",
                params![tag_id],
            )?;
        }

        Ok(())
    }


    fn remove_tag_from_file_by_name(&self, file_id: i64, tag_name: &str) -> Result<()> {
        // タグ名からタグIDを取得
        let tag_id: i64 = self.conn.query_row(
            "SELECT id FROM tags WHERE name = ?1",
            [tag_name],
            |row| row.get(0)
        )?;
        
        // 既存のremove_tag_from_file関数を呼び出し
        self.remove_tag_from_file(file_id, tag_id)
    }
    fn get_tags_for_file(&self, file_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.color, t.category, t.parent_tag_id, t.usage_count, t.created_at
             FROM tags t
             INNER JOIN file_tags ft ON t.id = ft.tag_id
             WHERE ft.file_id = ?1
             ORDER BY t.name ASC"
        )?;

        let tag_iter = stmt.query_map([file_id], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                parent_tag_id: row.get(4)?,
                usage_count: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;

        let mut tags = Vec::new();
        for tag in tag_iter {
            tags.push(tag?);
        }
        Ok(tags)
    }

    fn get_files_with_tags(&self) -> Result<Vec<FileWithTags>> {
        // 全ファイルを取得
        let mut file_stmt = self.conn.prepare(
            "SELECT id, file_path, file_name, file_size, file_hash, booth_product_id,
                    booth_shop_name, booth_product_name, booth_url, booth_price,
                    booth_thumbnail_path, encoding_info, created_at, updated_at, metadata
             FROM files ORDER BY created_at DESC"
        )?;

        let file_iter = file_stmt.query_map([], |row| {
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

        let mut files_with_tags = Vec::new();
        for file_result in file_iter {
            let file = file_result?;
            let file_id = file.id.unwrap_or(0);
            let tags = self.get_tags_for_file(file_id)?;
            
            files_with_tags.push(FileWithTags { file, tags });
        }

        Ok(files_with_tags)
    }

    fn get_files_with_tags_by_ids(&self, file_ids: &[i64]) -> Result<Vec<FileWithTags>> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = file_ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "SELECT id, file_path, file_name, file_size, file_hash, booth_product_id,
                    booth_shop_name, booth_product_name, booth_url, booth_price,
                    booth_thumbnail_path, encoding_info, created_at, updated_at, metadata
             FROM files 
             WHERE id IN ({})
             ORDER BY created_at DESC",
            placeholders.join(",")
        );

        let mut stmt = self.conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> = file_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        
        let file_iter = stmt.query_map(&params[..], |row| {
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

        let mut files_with_tags = Vec::new();
        for file_result in file_iter {
            let file = file_result?;
            let file_id = file.id.unwrap_or(0);
            let tags = self.get_tags_for_file(file_id)?;
            
            files_with_tags.push(FileWithTags { file, tags });
        }

        Ok(files_with_tags)
    }


    fn batch_add_tag_to_files(&self, file_ids: &[i64], tag_id: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let mut added_count = 0;

        for file_id in file_ids {
            // 重複をチェックしながら追加
            let affected_rows = tx.execute(
                "INSERT OR IGNORE INTO file_tags (file_id, tag_id, added_at) 
                 VALUES (?1, ?2, datetime('now'))",
                params![file_id, tag_id],
            )?;
            
            if affected_rows > 0 {
                added_count += 1;
            }
        }

        // タグの使用回数を実際に追加された数だけ増加
        if added_count > 0 {
            tx.execute(
                "UPDATE tags SET usage_count = usage_count + ?1 WHERE id = ?2",
                params![added_count, tag_id],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn batch_remove_tag_from_files(&self, file_ids: &[i64], tag_id: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let mut removed_count = 0;

        for file_id in file_ids {
            let affected_rows = tx.execute(
                "DELETE FROM file_tags WHERE file_id = ?1 AND tag_id = ?2",
                params![file_id, tag_id],
            )?;
            
            if affected_rows > 0 {
                removed_count += 1;
            }
        }

        // タグの使用回数を実際に削除された数だけ減少
        if removed_count > 0 {
            tx.execute(
                "UPDATE tags SET usage_count = MAX(0, usage_count - ?1) WHERE id = ?2",
                params![removed_count, tag_id],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    fn get_files_with_tags_paginated(&self, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileWithTags>, u32)> {
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

        let mut file_stmt = self.conn.prepare(&query)?;
        let file_iter = file_stmt.query_map([page_size, offset], |row| {
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

        let mut files_with_tags = Vec::new();
        for file_result in file_iter {
            let file = file_result?;
            let file_id = file.id.unwrap_or(0);
            let tags = self.get_tags_for_file(file_id)?;
            
            files_with_tags.push(FileWithTags { file, tags });
        }

        Ok((files_with_tags, total_count))
    }

    fn search_files_with_tags_by_tags_paginated(&self, tag_names: &[String], page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<FileWithTags>, u32)> {
        if tag_names.is_empty() {
            return Ok((Vec::new(), 0));
        }

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

        // タグ名のプレースホルダーを作成
        let tag_placeholders: Vec<String> = tag_names.iter().map(|_| "?".to_string()).collect();
        let tag_placeholders_str = tag_placeholders.join(",");

        // 指定されたタグを持つファイルの総数を取得
        let count_query = format!(
            "SELECT COUNT(DISTINCT f.id)
             FROM files f
             INNER JOIN file_tags ft ON f.id = ft.file_id
             INNER JOIN tags t ON ft.tag_id = t.id
             WHERE t.name IN ({})",
            tag_placeholders_str
        );

        let mut count_stmt = self.conn.prepare(&count_query)?;
        let tag_params: Vec<&dyn rusqlite::ToSql> = tag_names.iter().map(|name| name as &dyn rusqlite::ToSql).collect();
        let total_count: u32 = count_stmt.query_row(&tag_params[..], |row| row.get(0))?;

        // ページネーションされた検索結果を取得
        let search_query = format!(
            "SELECT DISTINCT f.id, f.file_path, f.file_name, f.file_size, f.file_hash, f.booth_product_id,
                    f.booth_shop_name, f.booth_product_name, f.booth_url, f.booth_price,
                    f.booth_thumbnail_path, f.encoding_info, f.created_at, f.updated_at, f.metadata
             FROM files f
             INNER JOIN file_tags ft ON f.id = ft.file_id
             INNER JOIN tags t ON ft.tag_id = t.id
             WHERE t.name IN ({})
             ORDER BY f.{} {} LIMIT ? OFFSET ?",
            tag_placeholders_str, sort_by, sort_order
        );

        let mut file_stmt = self.conn.prepare(&search_query)?;
        let mut search_params = tag_params;
        search_params.push(&page_size);
        search_params.push(&offset);

        let file_iter = file_stmt.query_map(&search_params[..], |row| {
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

        let mut files_with_tags = Vec::new();
        for file_result in file_iter {
            let file = file_result?;
            let file_id = file.id.unwrap_or(0);
            let tags = self.get_tags_for_file(file_id)?;
            
            files_with_tags.push(FileWithTags { file, tags });
        }

        Ok((files_with_tags, total_count))
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

        conn.execute(
            "CREATE TABLE tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL,
                category TEXT,
                parent_tag_id INTEGER,
                usage_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT,
                FOREIGN KEY (parent_tag_id) REFERENCES tags(id)
            )",
            [],
        ).unwrap();

        conn.execute(
            "CREATE TABLE file_tags (
                file_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                added_at TEXT DEFAULT (datetime('now')),
                PRIMARY KEY (file_id, tag_id),
                FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        ).unwrap();

        (db_file, conn)
    }

    fn create_test_file(conn: &Connection, name: &str) -> i64 {
        conn.execute(
            "INSERT INTO files (file_path, file_name, file_size, created_at) 
             VALUES (?1, ?2, 1024, datetime('now'))",
            params![format!("/test/{}", name), name],
        ).unwrap();
        conn.last_insert_rowid()
    }

    fn create_test_tag(conn: &Connection, name: &str, color: &str) -> i64 {
        conn.execute(
            "INSERT INTO tags (name, color, usage_count, created_at) 
             VALUES (?1, ?2, 0, datetime('now'))",
            params![name, color],
        ).unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn test_add_and_remove_tag_to_file() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteFileTagRepository::new(&conn);

        let file_id = create_test_file(&conn, "test.zip");
        let tag_id = create_test_tag(&conn, "VRChat", "#e74c3c");

        // Add tag to file
        repo.add_tag_to_file(file_id, tag_id).expect("Failed to add tag to file");

        // Verify tag was added
        let tags = repo.get_tags_for_file(file_id).expect("Failed to get tags for file");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "VRChat");

        // Verify usage count increased
        let updated_tag: Tag = conn.query_row(
            "SELECT id, name, color, category, parent_tag_id, usage_count, created_at FROM tags WHERE id = ?1",
            [tag_id],
            |row| Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                parent_tag_id: row.get(4)?,
                usage_count: row.get(5)?,
                created_at: row.get(6)?,
            })
        ).expect("Failed to get updated tag");
        assert_eq!(updated_tag.usage_count, 1);

        // Remove tag from file
        repo.remove_tag_from_file(file_id, tag_id).expect("Failed to remove tag from file");

        // Verify tag was removed
        let tags = repo.get_tags_for_file(file_id).expect("Failed to get tags for file");
        assert_eq!(tags.len(), 0);

        // Verify usage count decreased
        let updated_tag: Tag = conn.query_row(
            "SELECT id, name, color, category, parent_tag_id, usage_count, created_at FROM tags WHERE id = ?1",
            [tag_id],
            |row| Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                parent_tag_id: row.get(4)?,
                usage_count: row.get(5)?,
                created_at: row.get(6)?,
            })
        ).expect("Failed to get updated tag");
        assert_eq!(updated_tag.usage_count, 0);
    }

    #[test]
    fn test_get_files_with_tags() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteFileTagRepository::new(&conn);

        let file1_id = create_test_file(&conn, "file1.zip");
        let file2_id = create_test_file(&conn, "file2.zip");
        let tag1_id = create_test_tag(&conn, "Unity", "#3498db");
        let tag2_id = create_test_tag(&conn, "Avatar", "#e74c3c");

        // Add tags to files
        repo.add_tag_to_file(file1_id, tag1_id).expect("Failed to add tag1 to file1");
        repo.add_tag_to_file(file1_id, tag2_id).expect("Failed to add tag2 to file1");
        repo.add_tag_to_file(file2_id, tag1_id).expect("Failed to add tag1 to file2");

        // Get all files with tags
        let files_with_tags = repo.get_files_with_tags().expect("Failed to get files with tags");
        assert_eq!(files_with_tags.len(), 2);

        // file1 should have 2 tags
        let file1 = files_with_tags.iter().find(|f| f.file.file_name == "file1.zip").unwrap();
        assert_eq!(file1.tags.len(), 2);

        // file2 should have 1 tag
        let file2 = files_with_tags.iter().find(|f| f.file.file_name == "file2.zip").unwrap();
        assert_eq!(file2.tags.len(), 1);
    }


    #[test]
    fn test_batch_operations() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteFileTagRepository::new(&conn);

        let file1_id = create_test_file(&conn, "file1.zip");
        let file2_id = create_test_file(&conn, "file2.zip");
        let file3_id = create_test_file(&conn, "file3.zip");
        let tag_id = create_test_tag(&conn, "TestTag", "#3498db");

        let file_ids = vec![file1_id, file2_id, file3_id];

        // Batch add tag to files
        repo.batch_add_tag_to_files(&file_ids, tag_id)
            .expect("Failed to batch add tag to files");

        // Verify all files have the tag
        for file_id in &file_ids {
            let tags = repo.get_tags_for_file(*file_id).expect("Failed to get tags for file");
            assert_eq!(tags.len(), 1);
            assert_eq!(tags[0].name, "TestTag");
        }

        // Verify usage count is correct
        let updated_tag: Tag = conn.query_row(
            "SELECT id, name, color, category, parent_tag_id, usage_count, created_at FROM tags WHERE id = ?1",
            [tag_id],
            |row| Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                parent_tag_id: row.get(4)?,
                usage_count: row.get(5)?,
                created_at: row.get(6)?,
            })
        ).expect("Failed to get updated tag");
        assert_eq!(updated_tag.usage_count, 3);

        // Batch remove tag from first two files
        repo.batch_remove_tag_from_files(&file_ids[0..2], tag_id)
            .expect("Failed to batch remove tag from files");

        // Verify first two files no longer have the tag
        for file_id in &file_ids[0..2] {
            let tags = repo.get_tags_for_file(*file_id).expect("Failed to get tags for file");
            assert_eq!(tags.len(), 0);
        }

        // Verify third file still has the tag
        let tags = repo.get_tags_for_file(file_ids[2]).expect("Failed to get tags for file");
        assert_eq!(tags.len(), 1);

        // Verify usage count decreased correctly
        let updated_tag: Tag = conn.query_row(
            "SELECT id, name, color, category, parent_tag_id, usage_count, created_at FROM tags WHERE id = ?1",
            [tag_id],
            |row| Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                parent_tag_id: row.get(4)?,
                usage_count: row.get(5)?,
                created_at: row.get(6)?,
            })
        ).expect("Failed to get updated tag");
        assert_eq!(updated_tag.usage_count, 1);
    }
}