// TagRepository - タグ関連操作の責務を分離
// TDDでリファクタリング実施

use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use crate::database::Tag;

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
            "INSERT INTO tags (name, usage_count)
             VALUES (?1, ?2)",
            params![
                tag.name,
                tag.usage_count,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    fn find_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, usage_count, created_at, updated_at
             FROM tags WHERE name = ?1"
        )?;

        let tag_result = stmt.query_row([name], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                usage_count: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        }).optional()?;

        Ok(tag_result)
    }

    fn find_all(&self) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, usage_count, created_at, updated_at
             FROM tags ORDER BY usage_count DESC, name ASC"
        )?;

        let tag_iter = stmt.query_map([], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                usage_count: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        let mut tags = Vec::new();
        for tag in tag_iter {
            tags.push(tag?);
        }
        Ok(tags)
    }

    fn get_or_create(&self, name: &str, _color: Option<&str>) -> Result<Tag> {
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
            usage_count: 0,
            created_at: "".to_string(),
            updated_at: "".to_string(),
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
                "SELECT id, name, usage_count, created_at, updated_at
                 FROM tags ORDER BY usage_count DESC, name ASC LIMIT ? OFFSET ?"
            )
        } else {
            format!(
                "SELECT id, name, usage_count, created_at, updated_at
                 FROM tags ORDER BY {} {} LIMIT ? OFFSET ?",
                sort_by, sort_order
            )
        };

        let mut stmt = self.conn.prepare(&query)?;
        let tag_iter = stmt.query_map([page_size, offset], |row| {
            Ok(Tag {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                usage_count: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        let mut tags = Vec::new();
        for tag in tag_iter {
            tags.push(tag?);
        }

        Ok((tags, total_count))
    }

}

