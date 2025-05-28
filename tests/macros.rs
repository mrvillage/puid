use std::str::FromStr;

puid::puid!(UserId = "usr");

#[test]
fn test_new() {
    let user_id = UserId::new();
    assert!(user_id.as_str().starts_with("usr_"));
    for c in user_id.as_str().chars().skip(4) {
        println!("Character: {}", c);
        assert!(c.is_ascii_alphanumeric());
    }
}

#[test]
fn test_from_str() {
    let user_id = UserId::from_str("usr_45A0IQarTgXyiRM6VQ9YbX").unwrap();
    assert_eq!(user_id.as_str(), "usr_45A0IQarTgXyiRM6VQ9YbX");

    // Test invalid prefix
    assert!(UserId::from_str("valid_45A0IQarTgXyiRM6VQ9YbX").is_err());

    // Test invalid length
    assert!(UserId::from_str("usr_123").is_err());

    // Test invalid suffix character
    assert!(UserId::from_str("usr_12@34").is_err());
}
