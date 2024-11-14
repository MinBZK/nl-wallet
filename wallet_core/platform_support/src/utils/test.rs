use tokio::fs::File;
use tokio::fs::{self};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use crate::utils::PlatformUtilities;

// This utility function is used both by the Rust unit tests test for the "software" feature
// and by integration test performed from Android / iOS.
// This would normally fall under dev-dependencies, however we need it in the main binary
// for the Android / iOS integration test.
pub async fn get_and_verify_storage_path<K: PlatformUtilities>() {
    let original_message = "Hello, wallet!";
    let mut path = K::storage_path()
        .await
        .expect("Could not get storage path")
        .into_os_string()
        .into_string()
        .expect("Could not convert PathBuf to String");

    // Perform basic path sanity check
    assert!(!path.is_empty());
    assert!(path.starts_with('/'));

    // Create the test.txt file path
    path.push_str("/test.txt");

    // Write the [original_message] to test.txt
    let mut test_file = File::create(&path).await.expect("Could not create test.txt file");
    test_file
        .write_all(original_message.as_bytes())
        .await
        .expect("Could not write to file");
    test_file.flush().await.expect("Could not flush file");

    // Open the test.txt file and read the contents
    let mut test_file_ro = File::open(&path).await.expect("Could not open test.txt file");
    let mut contents = String::new();
    test_file_ro
        .read_to_string(&mut contents)
        .await
        .expect("Could not read test.txt");

    // Clean up and verify the file contents match the [original_message]
    fs::remove_file(&path).await.expect("Could not delete test.txt");

    assert_eq!(contents, original_message, "file contents should match written payload");
}
