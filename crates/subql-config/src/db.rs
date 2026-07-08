//! Postgres connection config — mirrors `@subql/node-core` `db/db.module.ts`,
//! which reads `DB_HOST`/`DB_PORT`/`DB_USER`/`DB_PASS`/`DB_DATABASE` from env
//! with the exact defaults reproduced below.

/// Postgres connection settings, populated from the same env vars as the TS node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl Default for DbConfig {
    fn default() -> Self {
        // Defaults copied verbatim from db.module.ts (lines 23-27).
        Self {
            host: "127.0.0.1".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database: "postgres".to_string(),
        }
    }
}

impl DbConfig {
    /// Load from the `DB_*` environment variables, falling back to the TS defaults.
    pub fn from_env() -> Self {
        let d = Self::default();
        Self {
            host: std::env::var("DB_HOST").unwrap_or(d.host),
            port: std::env::var("DB_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(d.port),
            username: std::env::var("DB_USER").unwrap_or(d.username),
            password: std::env::var("DB_PASS").unwrap_or(d.password),
            database: std::env::var("DB_DATABASE").unwrap_or(d.database),
        }
    }

    /// A libpq/tokio-postgres connection string.
    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.host, self.port, self.username, self.password, self.database
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_ts() {
        let d = DbConfig::default();
        assert_eq!(d.host, "127.0.0.1");
        assert_eq!(d.port, 5432);
        assert_eq!(d.username, "postgres");
        assert_eq!(d.database, "postgres");
    }

    #[test]
    fn connection_string_shape() {
        let s = DbConfig::default().connection_string();
        assert!(s.contains("host=127.0.0.1"));
        assert!(s.contains("dbname=postgres"));
    }
}
