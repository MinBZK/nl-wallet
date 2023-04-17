use std::fs;

use rand::distributions::{Alphanumeric, DistString};
use rusqlite::Connection;
use platform_support::utils::PlatformUtilities;
use platform_support::utils::software::SoftwareUtilities;

pub async fn get_or_create_db(db_name: &str) -> Connection {
    //Get path to database
    let storage_path = SoftwareUtilities::storage_path().expect("Could not get storage path");
    let db_path = storage_path.join(format!("{}.db", db_name));

    // Get db password
    let db_password = get_or_create_db_password(db_name).await;

    // Open db
    let conn = Connection::open(db_path).expect("Failed to open database");

    // Enable SQLCipher / Db Encryption
    let encrypt_statement = format!("PRAGMA key = '{}';", &db_password);
    conn.prepare(&*encrypt_statement).expect("Could not encrypt database");

    // return db connection
    conn
}

fn delete_db(db_name: &str) {
    //Get path to database
    let storage_path = SoftwareUtilities::storage_path().expect("Could not get storage path");
    let db_path = storage_path.join(format!("{}.db", db_name));
    if db_path.exists() { fs::remove_file(db_path).expect("Failed to delete database"); }
}

pub async fn get_or_create_db_password(alias: &str) -> String {
    // Get path to password file
    let storage_path = SoftwareUtilities::storage_path().expect("Could not get storage path");
    let pw_file_path = storage_path.join(format!("{}.pass", alias));

    if pw_file_path.exists() {
        // Open file
        let contents = fs::read_to_string(pw_file_path).expect("Could not read password.txt");
        // Decrypt password
        //TODO: Decrypt the contents
        contents
    } else {
        // Generate password
        let new_password = generate_db_password();
        //TODO: Encrypt the password
        fs::write(pw_file_path, &new_password).expect("Unable to write password.txt");
        new_password
    }
}

fn generate_db_password() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 24)
}

#[cfg(test)]
mod tests {
    use rusqlite::params;

    use super::*;

    struct Person {
        id: i32,
        name: String,
        data: Option<Vec<u8>>,
    }

    #[test]
    fn test_generate_password() {
        assert_eq!(24, generate_db_password().len())
    }

    #[tokio::test]
    async fn test_create_and_get_password() {
        let alias = "alias";
        let created_pass = get_or_create_db_password(alias).await;
        let fetched_pass = get_or_create_db_password(alias).await;
        assert_eq!(created_pass, fetched_pass)
    }

    #[tokio::test]
    async fn test_password_should_be_unique() {
        let alias = "pass1";
        let alias2 = "pass2";
        let pass1 = get_or_create_db_password(alias).await;
        let pass2 = get_or_create_db_password(alias2).await;
        assert_ne!(pass1, pass2)
    }

    #[tokio::test]
    async fn open_db() {
        let db_name = "test";

        // Make sure we always start clean, e.g. when previous test failed.
        delete_db(db_name);

        // Create a new (encrypted) database
        let conn = get_or_create_db(db_name).await;

        // Create a table for our [Person] model
        conn.execute(
            "CREATE TABLE person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            data  BLOB
        )", []).expect("Could not create table");

        // Create and insert our test Person
        let me = Person {
            id: 1337,
            name: "Willeke".to_string(),
            data: None,
        };
        conn.execute(
            "INSERT INTO person (id, name, data) VALUES (?1, ?2, ?3)",
            params![&me.id, &me.name, &me.data],
        ).expect("Could not insert person");

        // Query our person table for any [Person]s
        let mut stmt = conn.prepare("SELECT id, name, data FROM person")
            .expect("Could not execute select statement");

        // Map our query results back to our [Person] model
        let person_iter = stmt.query_map([], |row| {
            Ok(Person {
                id: row.get(0)?,
                name: row.get(1)?,
                data: row.get(2)?,
            })
        }).expect("Could not create iterator");

        // Verify our test [Person] was correctly inserted
        let mut person_count = 0;
        for person in person_iter {
            let result = person.unwrap();
            assert_eq!(1337, result.id);
            assert_eq!("Willeke", result.name);
            assert_eq!(None, result.data);
            person_count += 1;
        }

        // Verify we really checked our test person (and did not loop over empty iterator)
        assert_eq!(person_count, 1);

        // Clean up test db
        delete_db(db_name);
    }
}
