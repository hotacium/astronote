sqlx::migrate!(r#"
    CREATE TABLE IF NOT EXISTS notes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        absolute_path TEXT NOT NULL UNIQUE,
        next_datetime TEXT NOT NULL,
        scheduler JSON NOT NULL
    );
"#);