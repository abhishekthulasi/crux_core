use crate::models::{FactTag, Turn};
use rusqlite::{params, Connection, Result};

pub struct DbContext {
    conn: Connection,
}

impl DbContext {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = Self { conn };
        db.initialize_schema()?;
        Ok(db)
    }

    pub fn insert_turn(&self, turn: &Turn) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO turns (id, parent_turn_id, role, content) 
             VALUES (?1, ?2, ?3, ?4)",
            params![turn.id, turn.parent_turn_id, turn.role, turn.content,],
        )?;
        Ok(())
    }

    fn initialize_schema(&self) -> Result<()> {
        // MVP 1: Adjacency List for branching conversations
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS turns (
                id TEXT PRIMARY KEY,
                parent_turn_id TEXT,
                role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
                content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (parent_turn_id) REFERENCES turns (id) ON DELETE CASCADE
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_parent_turn ON turns(parent_turn_id)",
            [],
        )?;

        // MVP 4: Flat relational table for semantic memory (No JSON blobs)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS facts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                category TEXT NOT NULL CHECK(category IN ('fact', 'preference', 'nuance', 'unresolved_thread')),
                content TEXT NOT NULL,
                context TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_facts_category ON facts(category)",
            [],
        )?;

        Ok(())
    }

    /// Recursively fetches a branch of conversation from a specific leaf node up to the root.
    pub fn get_chat_branch(&self, active_turn_id: &str) -> Result<Vec<Turn>> {
        let mut stmt = self.conn.prepare(
            "WITH RECURSIVE chat_branch AS (
                SELECT id, parent_turn_id, role, content, created_at, 0 as depth
                FROM turns
                WHERE id = ?1
                
                UNION ALL
                
                SELECT t.id, t.parent_turn_id, t.role, t.content, t.created_at, cb.depth + 1
                FROM turns t
                INNER JOIN chat_branch cb ON t.id = cb.parent_turn_id
            )
            SELECT id, parent_turn_id, role, content, created_at FROM chat_branch ORDER BY depth DESC"
        )?;

        let turn_iter = stmt.query_map([active_turn_id], |row| {
            Ok(Turn {
                id: row.get(0)?,
                parent_turn_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        let mut branch = Vec::new();
        for turn in turn_iter {
            branch.push(turn?);
        }
        Ok(branch)
    }

    /// Inserts a validated fact struct directly into relational columns
    pub fn insert_fact(&self, fact: &FactTag, user_id: &str) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO facts (id, user_id, category, content, context) VALUES (?1, ?2, ?3, ?4, ?5)",
            [&id, user_id, &fact.category, &fact.content, &fact.context],
        )?;
        Ok(())
    }

    /// Fetches all turns that share the same parent as the given turn (Siblings)
    pub fn get_siblings(&self, turn_id: &str) -> rusqlite::Result<Vec<Turn>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, parent_turn_id, role, content, created_at 
             FROM turns 
             WHERE IFNULL(parent_turn_id, '') = (
                 SELECT IFNULL(parent_turn_id, '') FROM turns WHERE id = ?1
             )
             ORDER BY created_at ASC",
        )?;

        let turn_iter = stmt.query_map([turn_id], |row| {
            Ok(Turn {
                id: row.get(0)?,
                parent_turn_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        let mut siblings = Vec::new();
        for turn in turn_iter {
            siblings.push(turn?);
        }
        Ok(siblings)
    }

    /// Recursively traverses down children to find the most recent leaf node of a branch
    pub fn get_latest_leaf(&self, turn_id: &str) -> rusqlite::Result<String> {
        let mut stmt = self.conn.prepare(
            "WITH RECURSIVE descendant AS (
                SELECT id, created_at, 0 as depth FROM turns WHERE id = ?1
                UNION ALL
                SELECT t.id, t.created_at, d.depth + 1
                FROM turns t
                INNER JOIN descendant d ON t.parent_turn_id = d.id
            )
            SELECT id FROM descendant ORDER BY depth DESC, created_at DESC LIMIT 1",
        )?;

        let leaf_id: String = stmt.query_row([turn_id], |row| row.get(0))?;
        Ok(leaf_id)
    }
}
