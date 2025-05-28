use std::str::FromStr;

puid::puid!(TestId = "test");

#[test]
fn test_new() {
    let user_id = TestId::new();
    assert!(user_id.as_str().starts_with("test_"));
    for c in user_id.as_str().chars().skip(5) {
        assert!(c.is_ascii_alphanumeric());
    }
}

#[test]
fn test_from_str() {
    let user_id = TestId::from_str("test_45A0IQarTgXyiRM6VQ9YbX").unwrap();
    assert_eq!(user_id.as_str(), "test_45A0IQarTgXyiRM6VQ9YbX");

    // Test invalid prefix
    assert!(TestId::from_str("invalid_45A0IQarTgXyiRM6VQ9YbX").is_err());

    // Test invalid length
    assert!(TestId::from_str("test_123").is_err());

    // Test invalid suffix character
    assert!(TestId::from_str("test_12@34").is_err());
}

#[test]
#[cfg(feature = "serde")]
fn test_serde() {
    let user_id = TestId::new();
    let serialized = serde_json::to_string(&user_id).unwrap();
    assert_eq!(serialized, format!("\"{}\"", user_id.as_str()));

    let deserialized: TestId = serde_json::from_str(&serialized).unwrap();
    assert_eq!(user_id, deserialized);
}

#[test]
#[cfg(feature = "postgres")]
fn test_sqlx() {
    use sqlx::Connection;

    let Some(database_url) = std::env::var("DATABASE_URL").ok() else {
        eprintln!("DATABASE_URL environment variable is not set.");
        return;
    };
    let user_id = TestId::new();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let mut conn = sqlx::postgres::PgConnection::connect(&database_url)
            .await
            .unwrap();

        let _ = sqlx::query(
            "CREATE DOMAIN test_id AS VARCHAR(27)
            CHECK (
              VALUE ~ '^test_[0-9A-Za-z]{22}$'
            );",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS test (
                id test_id PRIMARY KEY,
                value integer NOT NULL
            );",
        )
        .execute(&mut conn)
        .await
        .unwrap();

        sqlx::query("INSERT INTO test (id, value) VALUES ($1, $2)")
            .bind(user_id)
            .bind(42)
            .execute(&mut conn)
            .await
            .unwrap();
    });
}

#[test]
fn test_create_domain() {
    let domain_sql = TestId::create_domain();
    assert_eq!(
        domain_sql,
        "CREATE DOMAIN test_id AS VARCHAR(27) CHECK (VALUE ~ '^test_[0-9A-Za-z]{22}$');"
    );
}
