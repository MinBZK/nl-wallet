use std::fs;
use std::str;

use rusqlite::Connection;

use platform_support::hw_keystore::software::SoftwareEncryptionKey;
use platform_support::utils::software::SoftwareUtilities;
use platform_support::utils::PlatformUtilities;
use wallet::database::password::delete_password;
use wallet::database::password::get_or_create_password;

pub async fn get_or_create_db(db_name: &str) -> Connection {
    //Get path to database
    //TODO: Migrate to generic PlatformUtilities
    let storage_path = SoftwareUtilities::storage_path().expect("Could not get storage path");
    let db_path = storage_path.join(format!("{}.db", db_name));

    // Get db password
    let db_password = get_or_create_password::<SoftwareEncryptionKey, SoftwareUtilities>(db_name)
        .await
        .expect("Could not get or create password");

    // Open db
    let conn = Connection::open(db_path).expect("Failed to open database");

    // Enable SQLCipher / Db Encryption
    let encrypt_statement = format!("PRAGMA key = '{}';", &db_password);
    conn.prepare(&encrypt_statement).expect("Could not encrypt database");

    // return db connection
    conn
}

async fn delete_db(db_name: &str) {
    // Get path to database
    //TODO: Migrate to generic PlatformUtilities
    let storage_path = SoftwareUtilities::storage_path().expect("Could not get storage path");
    let db_path = storage_path.join(format!("{}.db", db_name));
    if db_path.exists() {
        fs::remove_file(db_path).expect("Failed to delete database");
    }

    // Database password relies on same name password file, clean that up too.
    _ = delete_password::<SoftwareUtilities>(db_name)
        .await
        .expect("Could not delete password");
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

    #[tokio::test]
    async fn open_db() {
        let db_name = "test_db";

        // Make sure we start with a clean slate
        delete_db(db_name).await;

        // Create a new (encrypted) database
        let conn = get_or_create_db(db_name).await;

        // Create a table for our [Person] model
        conn.execute(
            "CREATE TABLE person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            data  BLOB
        )",
            [],
        )
        .expect("Could not create table");

        // Create and insert our test Person
        let me = Person {
            id: 1337,
            name: "Willeke".to_string(),
            data: None,
        };
        conn.execute(
            "INSERT INTO person (id, name, data) VALUES (?1, ?2, ?3)",
            params![&me.id, &me.name, &me.data],
        )
        .expect("Could not insert person");

        // Query our person table for any [Person]s
        let mut stmt = conn
            .prepare("SELECT id, name, data FROM person")
            .expect("Could not execute select statement");

        // Map our query results back to our [Person] model
        let person_iter = stmt
            .query_map([], |row| {
                Ok(Person {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    data: row.get(2)?,
                })
            })
            .expect("Could not create iterator");

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
    }
}
