pub fn connection_string(host: &str, db_name: &str, username: Option<&str>, password: Option<&str>) -> String {
    let username_password = username
        .map(|u| format!("{}{}@", u, password.map(|p| format!(":{}", p)).unwrap_or_default()))
        .unwrap_or_default();

    format!("postgres://{}{}/{}", username_password, host, db_name)
}

#[cfg(test)]
mod tests {
    use crate::postgres::connection_string;

    #[test]
    fn test_connection_string() {
        assert_eq!(
            connection_string("host", "db", Some("user"), Some("pwd")),
            "postgres://user:pwd@host/db"
        );
        assert_eq!(connection_string("host", "db", None, None), "postgres://host/db");
        assert_eq!(
            connection_string("host", "db", Some("user"), None),
            "postgres://user@host/db"
        );
        assert_eq!(connection_string("host", "db", None, Some("pwd")), "postgres://host/db");
    }
}
