use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use rusqlite::Connection;

pub struct TokenExtractor;

impl TokenExtractor {
    pub fn get_access_token() -> Result<Option<String>> {
        let db_path = Self::get_db_path()?;

        if !db_path.exists() {
            return Ok(None);
        }

        let conn = Connection::open(&db_path).context("Cannot open database")?;
        let mut stmt = conn
            .prepare("SELECT value FROM itemTable WHERE key = 'cursorAuth/accessToken'")
            .context("Failed to prepare SQL statement")?;

        let mut rows = stmt.query([]).context("Query execution failed")?;

        if let Some(row) = rows.next()? {
            let token: String = row.get(0)?;
            return Ok(Some(token));
        }

        Ok(None)
    }

    fn get_db_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            let appdata = dirs::data_local_dir().context("Cannot retrieve user APPDATA directory")?;
            Ok(PathBuf::from(appdata).join("Cursor").join("User").join("globalStorage").join("state.vscdb"))
        }

        #[cfg(target_os = "macos")]
        {
            let home = dirs::home_dir().context("Cannot retrieve user home directory")?;
            Ok(home
                .join("Library")
                .join("Application Support")
                .join("Cursor")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb"))
        }

        #[cfg(target_os = "linux")]
        {
            let home = dirs::home_dir().context("Cannot retrieve user home directory")?;
            Ok(home.join(".cursor").join("User").join("globalStorage").join("state.vscdb"))
        }
    }
}
