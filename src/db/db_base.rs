use rusqlite::Connection;
use std::path::{Path, PathBuf};

use crate::wx_core::utils::{wx_core_error, WxCoreError, WxCoreResult};

/// Base database handler
pub struct DBHandler {
    pub db_path: PathBuf,
    pub connection: Connection,
}

impl DBHandler {
    /// Create a new database handler
    pub fn new(db_path: impl AsRef<Path>) -> WxCoreResult<Self> {
        wx_core_error(|| {
            let db_path = db_path.as_ref();

            if !db_path.exists() {
                return Err(WxCoreError::InvalidPath(format!(
                    "Database file not found: {}",
                    db_path.display()
                )));
            }

            let connection = Connection::open(db_path)?;

            Ok(Self {
                db_path: db_path.to_path_buf(),
                connection,
            })
        })
    }

    /// Execute a SQL query and return the results as a vector of maps
    pub fn execute_query(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> WxCoreResult<Vec<serde_json::Value>> {
        wx_core_error(|| {
            let mut stmt = self.connection.prepare(sql)?;
            let column_names: Vec<String> = stmt
                .column_names()
                .into_iter()
                .map(|s| s.to_string())
                .collect();

            let rows = stmt.query_map(params, |row| {
                let mut map = serde_json::Map::new();

                for (i, name) in column_names.iter().enumerate() {
                    let value = match row.get_ref(i)? {
                        rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                        rusqlite::types::ValueRef::Integer(i) => {
                            serde_json::Value::Number(i.into())
                        }
                        rusqlite::types::ValueRef::Real(f) => {
                            if let Some(n) = serde_json::Number::from_f64(f) {
                                serde_json::Value::Number(n)
                            } else {
                                serde_json::Value::String(f.to_string())
                            }
                        }
                        rusqlite::types::ValueRef::Text(t) => {
                            serde_json::Value::String(String::from_utf8_lossy(t).to_string())
                        }
                        rusqlite::types::ValueRef::Blob(b) => {
                            serde_json::Value::String(format!("<BLOB: {} bytes>", b.len()))
                        }
                    };

                    map.insert(name.clone(), value);
                }

                Ok(serde_json::Value::Object(map))
            })?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }

            Ok(result)
        })
    }

    /// Execute a SQL query and return the first result
    pub fn execute_query_one(
        &self,
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> WxCoreResult<Option<serde_json::Value>> {
        wx_core_error(|| {
            let results = self.execute_query(sql, params)?;
            Ok(results.into_iter().next())
        })
    }

    /// Execute a SQL statement
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> WxCoreResult<usize> {
        wx_core_error(|| {
            let result = self.connection.execute(sql, params)?;
            Ok(result)
        })
    }

    /// Get the list of tables in the database
    pub fn get_tables(&self) -> WxCoreResult<Vec<String>> {
        wx_core_error(|| {
            let sql = "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name";
            let results = self.execute_query(sql, &[])?;

            let tables = results
                .into_iter()
                .filter_map(|row| {
                    if let serde_json::Value::Object(map) = row {
                        if let Some(serde_json::Value::String(name)) = map.get("name") {
                            return Some(name.clone());
                        }
                    }
                    None
                })
                .collect();

            Ok(tables)
        })
    }

    /// Get the schema of a table
    pub fn get_table_schema(&self, table_name: &str) -> WxCoreResult<Vec<String>> {
        wx_core_error(|| {
            let sql = format!("PRAGMA table_info({})", table_name);
            let results = self.execute_query(&sql, &[])?;

            let columns = results
                .into_iter()
                .filter_map(|row| {
                    if let serde_json::Value::Object(map) = row {
                        if let Some(serde_json::Value::String(name)) = map.get("name") {
                            return Some(name.clone());
                        }
                    }
                    None
                })
                .collect();

            Ok(columns)
        })
    }

    /// Close the database connection
    pub fn close(self) -> WxCoreResult<()> {
        wx_core_error(|| {
            drop(self.connection);
            Ok(())
        })
    }
}
