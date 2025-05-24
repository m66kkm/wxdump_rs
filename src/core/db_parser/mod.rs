// src/core/db_parser/mod.rs

pub mod micro_msg_parser; 
pub use micro_msg_parser::*; 

use anyhow::{Result, anyhow};
use rusqlite::{Result as RusqliteResult, types::Value};
use std::collections::HashMap;
use rusqlite::{Connection, OpenFlags, Error};
use std::path::PathBuf;

/// Opens a potentially SQLCipher encrypted SQLite database and sets the key and PRAGMAs.
/// This function now expects to operate on files that might still require SQLCipher PRAGMAs
/// to be correctly interpreted, even if page content is decrypted (due to preserved page reserved areas).
pub fn open_database(db_path: &PathBuf, key: &str) -> Result<Connection> {
    if !db_path.exists() {
        return Err(anyhow!("Database file not found: {:?}", db_path));
    }
    // Key can be empty if we expect SQLCipher to handle a db with decrypted content but SQLCipher structure.
    // if key.is_empty() {
    //     return Err(anyhow!("Database key is empty."));
    // }

    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
    println!("[DBParser] Opened database file: {:?}", db_path);

    // Set the key. If key is empty, SQLCipher might use a default or no encryption.
    let key_pragma_value = if key.is_empty() {
        "''".to_string() // Empty key for SQLCipher
    } else {
        format!("x'{}'", key)
    };
    
    match conn.pragma_update(None, "key", &key_pragma_value) {
        Ok(_) => println!("[DBParser] PRAGMA key = '{}' executed successfully.", key_pragma_value),
        Err(e) => {
            eprintln!("[DBParser] Failed to execute PRAGMA key = '{}': {}. Database might not be accessible.", key_pragma_value, e);
            return Err(anyhow!("Failed to set database key with PRAGMA key = '{}': {}", key_pragma_value, e));
        }
    }
    
    // Set other SQLCipher PRAGMAs that were found in the Python source or are common.
    // These might be necessary for SQLCipher to correctly interpret page structure even if content is decrypted.
    let pragma_batch = "PRAGMA cipher_page_size = 4096;\
                        PRAGMA kdf_iter = 64000;\
                        PRAGMA cipher_hmac_algorithm = HMAC_SHA1;\
                        PRAGMA cipher_kdf_algorithm = PBKDF2_HMAC_SHA1;\
                        PRAGMA cipher_compatibility = 1;"; // Changed to compatibility mode 1

    match conn.execute_batch(pragma_batch) {
        Ok(_) => println!("[DBParser] SQLCipher PRAGMAs (page_size, kdf_iter, compatibility, etc.) set successfully."),
        Err(e) => println!("[DBParser] Warning: Failed to set some SQLCipher PRAGMAs: {}. Proceeding anyway.", e),
    }

    // Verify readability by a simple query.
    match conn.query_row("SELECT count(*) FROM sqlite_master;", [], |row| row.get::<_, i32>(0)) {
        Ok(count) => {
            println!("[DBParser] Successfully queried sqlite_master, found {} entries. Key and PRAGMAs likely correct for file structure.", count);
            Ok(conn)
        }
        Err(e) => {
            eprintln!("[DBParser] Failed to query sqlite_master after setting key and PRAGMAs (key used: {}): {}", key_pragma_value, e);
            eprintln!("[DBParser] This indicates an issue with the key, PRAGMAs, or the DB file structure/corruption.");
            Err(anyhow!("Failed to verify database (key: '{}', path: {:?}) after PRAGMA settings: {}", key_pragma_value, db_path, e))
        }
    }
}

pub fn list_tables(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table';")?;
    let table_names = stmt.query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;
    Ok(table_names)
}
/// Connects to a standard (decrypted) SQLite database file.
///
/// # Arguments
///
/// * `db_path` - A path to the SQLite database file.
///
/// # Returns
///
/// * `Result&lt;rusqlite::Connection, rusqlite::Error&gt;` - Ok(Connection) if successful, Err(rusqlite::Error) otherwise.
pub fn connect_sqlite_db(db_path: &std::path::Path) -> std::result::Result<rusqlite::Connection, rusqlite::Error> {
    rusqlite::Connection::open(db_path)
}
/// Fetches all rows from a specified table in the SQLite database.
///
/// # Arguments
///
/// * `conn` - A reference to the `rusqlite::Connection`.
/// * `table_name` - The name of the table to fetch data from. Assumed to be trusted.
///
/// # Returns
///
/// * `RusqliteResult&lt;Vec&lt;HashMap&lt;String, Value&gt;&gt;&gt;` - A vector of HashMaps, where each HashMap represents a row
///   with column names as keys and column values as `rusqlite::types::Value`.
pub fn get_all_rows_from_table(
    conn: &Connection,
    table_name: &str,
) -> RusqliteResult<Vec<HashMap<String, Value>>> {
    // Construct the SQL query.
    // IMPORTANT: table_name is assumed to be trusted and not from direct user input
    // to prevent SQL injection. For this task, this assumption is acceptable.
    let query = format!("SELECT * FROM {}", table_name);
    let mut stmt = conn.prepare(&query)?;

    let mut rows = stmt.query_map([], |row| {
        let mut map = HashMap::new();
        let column_count = row.as_ref().column_count();
        for i in 0..column_count {
            let column_name = row.as_ref().column_name(i)?.to_string();
            let value = row.get(i)?;
            map.insert(column_name, value);
        }
        Ok(map)
    })?;

    let mut result_vec = Vec::new();
    for row_result in rows {
        result_vec.push(row_result?);
    }

    Ok(result_vec)
}