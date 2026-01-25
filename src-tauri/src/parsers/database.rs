use rusqlite::{Connection, Result as SqliteResult, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Player not found: {0}")]
    PlayerNotFound(String),
}

/// A player record from the players.db database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRecord {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub password_hash: String,
    pub x: f64,
    pub y: f64,
    pub z: i32,
    pub is_admin: bool,
    pub access_level: String,
    pub last_connection: Option<String>,
    pub steam_id: Option<String>,
    pub hours_played: f64,
    pub is_banned: bool,
}

/// Player whitelist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub steam_id: Option<String>,
}

/// Handler for the players.db SQLite database
pub struct PlayersDatabase {
    conn: Connection,
    path: String,
}

impl PlayersDatabase {
    /// Open an existing players.db file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DatabaseError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let conn = Connection::open(&path)?;

        Ok(Self {
            conn,
            path: path_str,
        })
    }

    /// Get all players from the database
    pub fn get_all_players(&self) -> Result<Vec<PlayerRecord>, DatabaseError> {
        // Try different table structures based on PZ version
        // Build 41 uses 'networkPlayers' or 'localPlayers' tables

        let tables = self.get_table_names()?;

        let mut players = Vec::new();

        // Try networkPlayers first (multiplayer)
        if tables.contains(&"networkPlayers".to_string()) {
            let mut stmt = self.conn.prepare(
                "SELECT * FROM networkPlayers"
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(PlayerRecord {
                    id: row.get(0).unwrap_or(0),
                    username: row.get(1).unwrap_or_default(),
                    display_name: row.get(2).unwrap_or_default(),
                    password_hash: row.get(3).unwrap_or_default(),
                    x: row.get(4).unwrap_or(0.0),
                    y: row.get(5).unwrap_or(0.0),
                    z: row.get(6).unwrap_or(0),
                    is_admin: row.get::<_, i32>(7).unwrap_or(0) != 0,
                    access_level: row.get(8).unwrap_or_default(),
                    last_connection: row.get(9).ok(),
                    steam_id: row.get(10).ok(),
                    hours_played: row.get(11).unwrap_or(0.0),
                    is_banned: row.get::<_, i32>(12).unwrap_or(0) != 0,
                })
            })?;

            for row in rows {
                if let Ok(player) = row {
                    players.push(player);
                }
            }
        }

        // Also try localPlayers (singleplayer)
        if tables.contains(&"localPlayers".to_string()) {
            let mut stmt = self.conn.prepare(
                "SELECT * FROM localPlayers"
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(PlayerRecord {
                    id: row.get(0).unwrap_or(0),
                    username: row.get(1).unwrap_or_else(|_| "local".to_string()),
                    display_name: row.get(2).unwrap_or_default(),
                    password_hash: String::new(),
                    x: row.get(3).unwrap_or(0.0),
                    y: row.get(4).unwrap_or(0.0),
                    z: row.get(5).unwrap_or(0),
                    is_admin: false,
                    access_level: String::new(),
                    last_connection: None,
                    steam_id: None,
                    hours_played: row.get(6).unwrap_or(0.0),
                    is_banned: false,
                })
            })?;

            for row in rows {
                if let Ok(player) = row {
                    players.push(player);
                }
            }
        }

        Ok(players)
    }

    /// Get a player by username
    pub fn get_player(&self, username: &str) -> Result<PlayerRecord, DatabaseError> {
        let tables = self.get_table_names()?;

        if tables.contains(&"networkPlayers".to_string()) {
            let mut stmt = self.conn.prepare(
                "SELECT * FROM networkPlayers WHERE username = ?1"
            )?;

            if let Ok(player) = stmt.query_row(params![username], |row| {
                Ok(PlayerRecord {
                    id: row.get(0).unwrap_or(0),
                    username: row.get(1).unwrap_or_default(),
                    display_name: row.get(2).unwrap_or_default(),
                    password_hash: row.get(3).unwrap_or_default(),
                    x: row.get(4).unwrap_or(0.0),
                    y: row.get(5).unwrap_or(0.0),
                    z: row.get(6).unwrap_or(0),
                    is_admin: row.get::<_, i32>(7).unwrap_or(0) != 0,
                    access_level: row.get(8).unwrap_or_default(),
                    last_connection: row.get(9).ok(),
                    steam_id: row.get(10).ok(),
                    hours_played: row.get(11).unwrap_or(0.0),
                    is_banned: row.get::<_, i32>(12).unwrap_or(0) != 0,
                })
            }) {
                return Ok(player);
            }
        }

        Err(DatabaseError::PlayerNotFound(username.to_string()))
    }

    /// Update a player's display name
    pub fn update_display_name(&self, username: &str, new_name: &str) -> Result<(), DatabaseError> {
        let tables = self.get_table_names()?;

        if tables.contains(&"networkPlayers".to_string()) {
            self.conn.execute(
                "UPDATE networkPlayers SET displayName = ?1 WHERE username = ?2",
                params![new_name, username],
            )?;
        }

        Ok(())
    }

    /// Update a player's position
    pub fn update_position(&self, username: &str, x: f64, y: f64, z: i32) -> Result<(), DatabaseError> {
        let tables = self.get_table_names()?;

        if tables.contains(&"networkPlayers".to_string()) {
            self.conn.execute(
                "UPDATE networkPlayers SET x = ?1, y = ?2, z = ?3 WHERE username = ?4",
                params![x, y, z, username],
            )?;
        }

        Ok(())
    }

    /// Set a player's admin status
    pub fn set_admin(&self, username: &str, is_admin: bool) -> Result<(), DatabaseError> {
        let tables = self.get_table_names()?;

        if tables.contains(&"networkPlayers".to_string()) {
            let admin_val = if is_admin { 1 } else { 0 };
            let access = if is_admin { "admin" } else { "" };

            self.conn.execute(
                "UPDATE networkPlayers SET admin = ?1, accessLevel = ?2 WHERE username = ?3",
                params![admin_val, access, username],
            )?;
        }

        Ok(())
    }

    /// Ban/unban a player
    pub fn set_banned(&self, username: &str, is_banned: bool) -> Result<(), DatabaseError> {
        let tables = self.get_table_names()?;

        if tables.contains(&"networkPlayers".to_string()) {
            let ban_val = if is_banned { 1 } else { 0 };

            self.conn.execute(
                "UPDATE networkPlayers SET banned = ?1 WHERE username = ?2",
                params![ban_val, username],
            )?;
        }

        Ok(())
    }

    /// Delete a player from the database
    pub fn delete_player(&self, username: &str) -> Result<(), DatabaseError> {
        let tables = self.get_table_names()?;

        if tables.contains(&"networkPlayers".to_string()) {
            self.conn.execute(
                "DELETE FROM networkPlayers WHERE username = ?1",
                params![username],
            )?;
        }

        if tables.contains(&"localPlayers".to_string()) {
            self.conn.execute(
                "DELETE FROM localPlayers WHERE username = ?1",
                params![username],
            )?;
        }

        Ok(())
    }

    /// Get whitelist entries
    pub fn get_whitelist(&self) -> Result<Vec<WhitelistEntry>, DatabaseError> {
        let tables = self.get_table_names()?;
        let mut entries = Vec::new();

        if tables.contains(&"whitelist".to_string()) {
            let mut stmt = self.conn.prepare("SELECT * FROM whitelist")?;

            let rows = stmt.query_map([], |row| {
                Ok(WhitelistEntry {
                    id: row.get(0).unwrap_or(0),
                    username: row.get(1).unwrap_or_default(),
                    password_hash: row.get(2).unwrap_or_default(),
                    is_admin: row.get::<_, i32>(3).unwrap_or(0) != 0,
                    steam_id: row.get(4).ok(),
                })
            })?;

            for row in rows {
                if let Ok(entry) = row {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    /// Add a player to whitelist
    pub fn add_to_whitelist(&self, username: &str, password_hash: &str, is_admin: bool) -> Result<(), DatabaseError> {
        let tables = self.get_table_names()?;

        if tables.contains(&"whitelist".to_string()) {
            let admin_val = if is_admin { 1 } else { 0 };

            self.conn.execute(
                "INSERT OR REPLACE INTO whitelist (username, password, admin) VALUES (?1, ?2, ?3)",
                params![username, password_hash, admin_val],
            )?;
        }

        Ok(())
    }

    /// Remove a player from whitelist
    pub fn remove_from_whitelist(&self, username: &str) -> Result<(), DatabaseError> {
        let tables = self.get_table_names()?;

        if tables.contains(&"whitelist".to_string()) {
            self.conn.execute(
                "DELETE FROM whitelist WHERE username = ?1",
                params![username],
            )?;
        }

        Ok(())
    }

    /// Get all table names in the database
    fn get_table_names(&self) -> Result<Vec<String>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table'"
        )?;

        let rows = stmt.query_map([], |row| {
            row.get(0)
        })?;

        let mut tables = Vec::new();
        for row in rows {
            if let Ok(name) = row {
                tables.push(name);
            }
        }

        Ok(tables)
    }

    /// Get database schema info for debugging
    pub fn get_schema_info(&self) -> Result<Vec<(String, String)>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT name, sql FROM sqlite_master WHERE type='table'"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
            ))
        })?;

        let mut schema = Vec::new();
        for row in rows {
            if let Ok(info) = row {
                schema.push(info);
            }
        }

        Ok(schema)
    }

    /// Get the database file path
    pub fn path(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_record_default() {
        let player = PlayerRecord {
            id: 1,
            username: "test".to_string(),
            display_name: "Test Player".to_string(),
            password_hash: String::new(),
            x: 10000.0,
            y: 10000.0,
            z: 0,
            is_admin: false,
            access_level: String::new(),
            last_connection: None,
            steam_id: None,
            hours_played: 0.0,
            is_banned: false,
        };

        assert_eq!(player.username, "test");
    }
}
