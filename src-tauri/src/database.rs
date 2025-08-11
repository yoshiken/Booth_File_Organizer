use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileRecord {
    pub id: Option<i64>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub modified_time: i64,
    pub created_at: String,
    pub updated_at: String,
    pub product_id: Option<String>,
    pub product_name: Option<String>,
    pub author_name: Option<String>,
    pub price: Option<i32>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub product_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: Option<i64>,
    pub name: String,
    pub usage_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileWithTags {
    pub file: FileRecord,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileUpdateFields {
    pub product_id: Option<String>,
    pub product_name: Option<String>,
    pub author_name: Option<String>,
    pub price: Option<i32>,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub product_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchStatistics {
    pub processed: usize,
    pub updated: usize,
    pub errors: usize,
}

pub struct DatabaseRefactored {
    conn: Connection,
}

impl DatabaseRefactored {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = DatabaseRefactored { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
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
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT UNIQUE NOT NULL,
                usage_count INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS file_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (file_id) REFERENCES files (id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE,
                UNIQUE(file_id, tag_id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn add_file(&self, file: FileRecord) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO files 
             (file_path, file_name, file_size, modified_time, product_id, product_name, 
              author_name, price, description, thumbnail_url, product_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        )?;
        
        stmt.execute(rusqlite::params![
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
        ])?;
        
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_files(&self) -> Result<Vec<FileRecord>> {
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

    pub fn update_file(&self, id: i64, updates: FileUpdateFields) -> Result<()> {
        self.conn.execute(
            "UPDATE files SET 
             product_id = ?1, product_name = ?2, author_name = ?3,
             price = ?4, description = ?5, thumbnail_url = ?6,
             product_url = ?7, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?8",
            rusqlite::params![
                updates.product_id,
                updates.product_name,
                updates.author_name,
                updates.price,
                updates.description,
                updates.thumbnail_url,
                updates.product_url,
                id
            ],
        )?;
        Ok(())
    }

    pub fn delete_file(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM files WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn add_tag(&self, name: &str) -> Result<i64> {
        let mut stmt = self.conn.prepare(
            "INSERT OR IGNORE INTO tags (name) VALUES (?1)"
        )?;
        stmt.execute([name])?;
        
        let mut stmt = self.conn.prepare("SELECT id FROM tags WHERE name = ?1")?;
        let tag_id: i64 = stmt.query_row([name], |row| row.get(0))?;
        
        Ok(tag_id)
    }

    pub fn get_all_tags(&self) -> Result<Vec<Tag>> {
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

    pub fn add_file_tag(&self, file_id: i64, tag_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?1, ?2)",
            [file_id, tag_id],
        )?;
        
        // Update usage count
        self.conn.execute(
            "UPDATE tags SET usage_count = (
                SELECT COUNT(*) FROM file_tags WHERE tag_id = ?1
            ) WHERE id = ?1",
            [tag_id],
        )?;
        
        Ok(())
    }

    pub fn get_files_with_tags(&self) -> Result<Vec<FileWithTags>> {
        let files = self.get_all_files()?;
        let mut files_with_tags = Vec::new();
        
        for file in files {
            let file_id = file.id.unwrap();
            let tags = self.get_tags_for_file(file_id)?;
            files_with_tags.push(FileWithTags { file, tags });
        }
        
        Ok(files_with_tags)
    }

    pub fn get_tags_for_file(&self, file_id: i64) -> Result<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, t.usage_count, t.created_at, t.updated_at
             FROM tags t
             JOIN file_tags ft ON t.id = ft.tag_id
             WHERE ft.file_id = ?1
             ORDER BY t.name"
        )?;
        
        let tag_iter = stmt.query_map([file_id], |row| {
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

    pub fn recalculate_usage_counts(&self) -> Result<()> {
        self.conn.execute(
            "UPDATE tags SET usage_count = (
                SELECT COUNT(*) FROM file_tags WHERE tag_id = tags.id
            )",
            [],
        )?;
        Ok(())
    }

    pub fn get_file_count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM files", 
            [], 
            |row| row.get(0)
        )?;
        Ok(count as usize)
    }

    pub fn get_tag_count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tags", 
            [], 
            |row| row.get(0)
        )?;
        Ok(count as usize)
    }
}