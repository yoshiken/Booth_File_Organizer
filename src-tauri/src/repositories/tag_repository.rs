// TagRepository - タグ関連操作の責務を分離
// TDDでリファクタリング実施

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use crate::database_refactored::Tag;
use crate::config::tags;

/// タグ操作の責務を持つRepository trait
pub trait TagRepository {
    fn insert(&self, tag: &Tag) -> Result<i64>;
    fn find_by_name(&self, name: &str) -> Result<Option<Tag>>;
    fn find_all(&self) -> Result<Vec<Tag>>;
    fn get_or_create(&self, name: &str, color: Option<&str>) -> Result<Tag>;
    fn recalculate_usage_count(&self) -> Result<()>;
    
    // 新しいページネーションメソッド
    fn find_all_paginated(&self, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<Tag>, u32)>;
}

/// SQLite実装のTagRepository
pub struct SqliteTagRepository<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteTagRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
}

impl<'a> TagRepository for SqliteTagRepository<'a> {
    fn insert(&self, tag: &Tag) -> Result<i64> {
        let _id = self.conn.execute(
            "INSERT INTO tags (name, color, category, parent_tag_id, usage_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
            params![
                tag.name,
                tag.color,
                tag.category,
                tag.parent_tag_id,
                tag.usage_count,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    fn find_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, category, parent_tag_id, usage_count, created_at
             FROM tags WHERE name = ?1"
        )?;

        let tag_result = stmt.query_row([name], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                color: row.get(2)?,
                category: row.get(3)?,
                parent_tag_id: row.get(4)?,
                usage_count: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).optional()?;

        Ok(tag_result)
    }

    fn find_all(&self) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, color, category, parent_tag_id, usage_count, created_at
             FROM tags ORDER BY usage_count DESC, name ASC"
        )?;

        let tag_iter = stmt.query_map([], |row| {
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

    fn get_or_create(&self, name: &str, color: Option<&str>) -> Result<Tag> {
        // 空文字列や空白のみのタグ名は許可しない
        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("タグ名は空にできません"));
        }

        // 既存のタグを検索
        if let Some(existing_tag) = self.find_by_name(name)? {
            return Ok(existing_tag);
        }

        // タグが存在しない場合は新規作成
        let new_tag = Tag {
            id: None,
            name: name.to_string(),
            color: color.unwrap_or(tags::ALTERNATIVE_DEFAULT_COLOR).to_string(), // デフォルトカラー
            category: None,
            parent_tag_id: None,
            usage_count: 0,
            created_at: None,
        };

        let _tag_id = self.insert(&new_tag)?;
        
        // 作成されたタグを返す
        self.find_by_name(name)?
            .ok_or_else(|| anyhow::anyhow!("Failed to retrieve created tag"))
    }

    fn recalculate_usage_count(&self) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        
        // すべてのタグのusage_countを0にリセット
        tx.execute("UPDATE tags SET usage_count = 0", [])?;
        
        // 実際のfile_tags関係からカウントを計算して更新
        tx.execute(
            "UPDATE tags SET usage_count = (
                SELECT COUNT(*) 
                FROM file_tags 
                WHERE file_tags.tag_id = tags.id
            )",
            [],
        )?;
        
        // usage_count が 0 になったタグを自動削除
        tx.execute("DELETE FROM tags WHERE usage_count = 0", [])?;
        
        tx.commit()?;
        Ok(())
    }

    fn find_all_paginated(&self, page: u32, page_size: u32, sort_by: Option<&str>, sort_order: Option<&str>) -> Result<(Vec<Tag>, u32)> {
        let sort_by = sort_by.unwrap_or("usage_count");
        let sort_order = sort_order.unwrap_or("desc");
        let offset = (page.saturating_sub(1)) * page_size;

        // 検証: sort_byは許可されたカラムのみ
        let allowed_columns = ["id", "name", "usage_count", "created_at"];
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
            "SELECT COUNT(*) FROM tags",
            [],
            |row| row.get(0)
        )?;

        // デフォルト以外のソート順序の場合の特別処理
        let query = if sort_by == "usage_count" && sort_order == "DESC" {
            // デフォルトのソート順序の場合、セカンダリソートにnameを追加
            format!(
                "SELECT id, name, color, category, parent_tag_id, usage_count, created_at
                 FROM tags ORDER BY usage_count DESC, name ASC LIMIT ? OFFSET ?"
            )
        } else {
            format!(
                "SELECT id, name, color, category, parent_tag_id, usage_count, created_at
                 FROM tags ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, sort_order
            )
        };

        let mut stmt = self.conn.prepare(&query)?;
        let tag_iter = stmt.query_map([page_size, offset], |row| {
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

        Ok((tags, total_count))
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

        // file_tags テーブルも作成（recalculate_usage_count テスト用）
        conn.execute(
            "CREATE TABLE file_tags (
                file_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                added_at TEXT DEFAULT (datetime('now')),
                PRIMARY KEY (file_id, tag_id),
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        ).unwrap();

        (db_file, conn)
    }

    #[test]
    fn test_insert_and_find_by_name() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteTagRepository::new(&conn);

        let test_tag = Tag {
            id: None,
            name: "VRChat".to_string(),
            color: "#e74c3c".to_string(),
            category: Some("Platform".to_string()),
            parent_tag_id: None,
            usage_count: 0,
            created_at: None,
        };

        // Insert test
        let tag_id = repo.insert(&test_tag).expect("Failed to insert tag");
        assert!(tag_id > 0);

        // Find by name test
        let retrieved_tag = repo.find_by_name("VRChat").expect("Failed to find tag");
        assert!(retrieved_tag.is_some());
        
        let tag = retrieved_tag.unwrap();
        assert_eq!(tag.name, "VRChat");
        assert_eq!(tag.color, "#e74c3c");
        assert_eq!(tag.category, Some("Platform".to_string()));
        assert_eq!(tag.usage_count, 0);
    }

    #[test]
    fn test_find_all() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteTagRepository::new(&conn);

        // Initially empty
        let tags = repo.find_all().expect("Failed to get all tags");
        assert_eq!(tags.len(), 0);

        // Insert two tags
        let tag1 = Tag {
            id: None,
            name: "Unity".to_string(),
            color: "#3498db".to_string(),
            category: None,
            parent_tag_id: None,
            usage_count: 5,
            created_at: None,
        };

        let tag2 = Tag {
            id: None,
            name: "Blender".to_string(),
            color: "#f39c12".to_string(),
            category: None,
            parent_tag_id: None,
            usage_count: 3,
            created_at: None,
        };

        repo.insert(&tag1).expect("Failed to insert tag1");
        repo.insert(&tag2).expect("Failed to insert tag2");

        // Should find both tags, sorted by usage_count DESC
        let tags = repo.find_all().expect("Failed to get all tags");
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name, "Unity"); // Higher usage_count first
        assert_eq!(tags[1].name, "Blender");
    }

    #[test]
    fn test_get_or_create_existing() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteTagRepository::new(&conn);

        // Create a tag first
        let original_tag = Tag {
            id: None,
            name: "Avatar".to_string(),
            color: "#9b59b6".to_string(),
            category: None,
            parent_tag_id: None,
            usage_count: 1,
            created_at: None,
        };

        repo.insert(&original_tag).expect("Failed to insert tag");

        // get_or_create should return existing tag
        let retrieved_tag = repo.get_or_create("Avatar", Some("#different_color"))
            .expect("Failed to get or create tag");
        
        assert_eq!(retrieved_tag.name, "Avatar");
        assert_eq!(retrieved_tag.color, "#9b59b6"); // Original color, not the new one
        assert_eq!(retrieved_tag.usage_count, 1);
    }

    #[test]
    fn test_get_or_create_new() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteTagRepository::new(&conn);

        // get_or_create should create new tag
        let new_tag = repo.get_or_create("NewTag", Some("#custom_color"))
            .expect("Failed to get or create tag");
        
        assert_eq!(new_tag.name, "NewTag");
        assert_eq!(new_tag.color, "#custom_color");
        assert_eq!(new_tag.usage_count, 0);

        // Should be able to find it now
        let found_tag = repo.find_by_name("NewTag").expect("Failed to find created tag");
        assert!(found_tag.is_some());
    }

    #[test]
    fn test_get_or_create_default_color() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteTagRepository::new(&conn);

        // get_or_create without color should use default
        let new_tag = repo.get_or_create("DefaultColorTag", None)
            .expect("Failed to get or create tag");
        
        assert_eq!(new_tag.name, "DefaultColorTag");
        assert_eq!(new_tag.color, "#3498db"); // Default color
    }

    #[test]
    fn test_recalculate_usage_count() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteTagRepository::new(&conn);

        // Create tags
        let tag1 = Tag {
            id: None,
            name: "Tag1".to_string(),
            color: "#3498db".to_string(),
            category: None,
            parent_tag_id: None,
            usage_count: 100, // Wrong count
            created_at: None,
        };

        let tag2 = Tag {
            id: None,
            name: "Tag2".to_string(),
            color: "#e74c3c".to_string(),
            category: None,
            parent_tag_id: None,
            usage_count: 200, // Wrong count
            created_at: None,
        };

        let tag1_id = repo.insert(&tag1).expect("Failed to insert tag1");
        let tag2_id = repo.insert(&tag2).expect("Failed to insert tag2");

        // Simulate file_tags relationships
        conn.execute(
            "INSERT INTO file_tags (file_id, tag_id) VALUES (1, ?1), (2, ?1), (3, ?2)",
            params![tag1_id, tag2_id]
        ).expect("Failed to insert file_tags");

        // Recalculate usage count
        repo.recalculate_usage_count().expect("Failed to recalculate usage count");

        // Check that counts are corrected
        let updated_tag1 = repo.find_by_name("Tag1").expect("Failed to find tag1").unwrap();
        let updated_tag2 = repo.find_by_name("Tag2").expect("Failed to find tag2").unwrap();

        assert_eq!(updated_tag1.usage_count, 2); // Should be 2 (files 1 and 2)
        assert_eq!(updated_tag2.usage_count, 1); // Should be 1 (file 3)
    }


    #[test]
    fn test_recalculate_usage_count_with_auto_deletion() {
        let (_db_file, conn) = create_test_db();
        let repo = SqliteTagRepository::new(&conn);

        // Create tags
        let tag1 = Tag {
            id: None,
            name: "WillBeUsed".to_string(),
            color: "#3498db".to_string(),
            category: None,
            parent_tag_id: None,
            usage_count: 100, // Wrong count, will be corrected
            created_at: None,
        };

        let tag2 = Tag {
            id: None,
            name: "WillBeDeleted".to_string(),
            color: "#e74c3c".to_string(),
            category: None,
            parent_tag_id: None,
            usage_count: 200, // Wrong count, will be set to 0 and deleted
            created_at: None,
        };

        let tag1_id = repo.insert(&tag1).expect("Failed to insert tag1");
        let _tag2_id = repo.insert(&tag2).expect("Failed to insert tag2");

        // Create file_tags relationship for tag1 only
        conn.execute(
            "INSERT INTO file_tags (file_id, tag_id) VALUES (1, ?1), (2, ?1)",
            params![tag1_id]
        ).expect("Failed to insert file_tags");

        // Initially both tags exist
        let all_tags = repo.find_all().expect("Failed to get all tags");
        assert_eq!(all_tags.len(), 2);

        // Recalculate usage count (includes auto-deletion)
        repo.recalculate_usage_count().expect("Failed to recalculate usage count");

        // Only tag1 should remain with correct count
        let remaining_tags = repo.find_all().expect("Failed to get remaining tags");
        assert_eq!(remaining_tags.len(), 1);
        assert_eq!(remaining_tags[0].name, "WillBeUsed");
        assert_eq!(remaining_tags[0].usage_count, 2);

        // tag2 should be deleted
        let deleted_tag_result = repo.find_by_name("WillBeDeleted").expect("Failed to search for deleted tag");
        assert!(deleted_tag_result.is_none());
    }
}