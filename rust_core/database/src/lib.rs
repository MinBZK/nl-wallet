use std::env::temp_dir;
use std::fs;
use std::path::{Path, PathBuf};

use rand::distributions::{Alphanumeric, DistString};
use rusqlite::Connection;

pub async fn get_or_create_db() -> Connection {
    //Get path to database
    //PlatformUtilities::storage_path().expect("Could not get storage path")
    let storage_path = temp_dir().to_str().expect("Could not convert to str").to_string();
    let db_file_path = format!("{}wallet.db", storage_path);
    // let sqlite_path = format!("sqlite://{}?mode=rwc", db_file_path);
    let sqlite_path = "sqlite://test.db?mode=rwc";

    // Get db password
    let db_password = get_or_create_db_password().await;
    // Open db
    // let conn = Connection::open(sqlite_path).expect("Could not open database");
    let path: PathBuf = "./test.db".into();
    let conn = Connection::open(path).expect("Failed to open database");
    let encrypt_statement = format!("PRAGMA key = '{}';", &db_password);
    // conn.execute(&*encrypt_statement, []).expect("Could not encrypt database");
    // conn.execute("PRAGMA key = 'supersecret';", []).expect("Could not encrypt database");
    // return db connection
    conn
}

fn delete_db() {
    let path: PathBuf = "./test.db".into();
    if path.exists() {
        fs::remove_file(path).expect("Could not delete test_db");
    }
}

pub async fn get_or_create_db_password() -> String {
    // Get path to password file
    let mut storage_path: PathBuf = temp_dir(); //TODO: Get path from platform_utils
    storage_path.push("password.txt");

    let pw_file_exists = storage_path.exists();
    // println!(storage_path);
    if pw_file_exists {
        // Open file
        let contents = fs::read_to_string(storage_path).expect("Could not read password.txt");
        // Decrypt password
        //TODO: Decrypt the contents
        contents
    } else {
        // Generate password
        let new_password = generate_db_password();
        //TODO: Encrypt the password
        fs::write(storage_path, &new_password).expect("Unable to write password.txt");
        new_password
    }
}

fn generate_db_password() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 24)
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
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
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_generate_password() {
        assert_eq!(24, generate_db_password().len())
    }

    #[tokio::test]
    async fn test_get_password() {
        let created_pass = get_or_create_db_password().await;
        println!("Created password: {}", created_pass);
        let fetched_pass = get_or_create_db_password().await;
        assert_eq!(created_pass, fetched_pass)
    }

    #[tokio::test]
    async fn open_db() {
        // Make sure we always start clean.
        delete_db();

        // Create a new (encrypted) database
        let conn = get_or_create_db().await;

        // Create a table for our [Person] model
        conn.execute(
            "CREATE TABLE person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            data  BLOB
        )", params![]).expect("Could not create table");

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
        let mut stmt = conn.prepare("SELECT id, name, data FROM person").expect("Could not execute select statement");

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
        delete_db();
    }
}
