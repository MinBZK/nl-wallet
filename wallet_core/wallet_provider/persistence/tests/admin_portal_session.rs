use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use crypto::utils::random_string;
use db_test::DbSetup;
use wallet_provider_domain::model::admin_portal_session::AdminPortalLoginAttempt;
use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
use wallet_provider_persistence::admin_portal_session;
use wallet_provider_persistence::test::db_from_setup;

/// Fixed instant used in place of `Utc::now()`. Postgres `timestamptz` only stores microsecond precision, so
/// a real "now" (which carries nanoseconds) would not compare equal to what gets read back from the database.
fn now() -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_login_attempt() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let state = random_string(32);
    let now = now();

    // Not present before insertion.
    let result = admin_portal_session::take_login_attempt(&db, &state).await.unwrap();
    assert!(result.is_none());

    admin_portal_session::insert_login_attempt(
        &db,
        state.clone(),
        AdminPortalLoginAttempt {
            nonce: "nonce".to_string(),
            code_verifier: "verifier".to_string(),
            created_at: now,
        },
    )
    .await
    .expect("should insert login attempt");

    // Taking it removes it and returns its contents.
    let login_attempt = admin_portal_session::take_login_attempt(&db, &state)
        .await
        .unwrap()
        .expect("login attempt should be present");
    assert_eq!(login_attempt.nonce, "nonce");
    assert_eq!(login_attempt.code_verifier, "verifier");
    assert_eq!(login_attempt.created_at, now);

    let result = admin_portal_session::take_login_attempt(&db, &state).await.unwrap();
    assert!(result.is_none(), "login attempt should have been removed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_cleanup_expired_login_attempts() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let old_state = random_string(32);
    let recent_state = random_string(32);
    let now = now();

    admin_portal_session::insert_login_attempt(
        &db,
        old_state.clone(),
        AdminPortalLoginAttempt {
            nonce: "nonce".to_string(),
            code_verifier: "verifier".to_string(),
            created_at: now - TimeDelta::minutes(20),
        },
    )
    .await
    .unwrap();
    admin_portal_session::insert_login_attempt(
        &db,
        recent_state.clone(),
        AdminPortalLoginAttempt {
            nonce: "nonce".to_string(),
            code_verifier: "verifier".to_string(),
            created_at: now,
        },
    )
    .await
    .unwrap();

    admin_portal_session::cleanup_expired_login_attempts(&db, now - TimeDelta::minutes(10))
        .await
        .expect("should clean up expired login attempts");

    assert!(
        admin_portal_session::take_login_attempt(&db, &old_state)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        admin_portal_session::take_login_attempt(&db, &recent_state)
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_user_session_and_take() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let session_id = random_string(32);
    let now = now();

    let expires_at = now + TimeDelta::minutes(15);
    admin_portal_session::insert_user_session(
        &db,
        session_id.clone(),
        AdminPortalUserSession {
            display_name: "Jane Doe".to_string(),
            roles: vec!["privilege_admin".to_string()],
            id_token: "id_token".to_string(),
            expires_at,
        },
    )
    .await
    .expect("should insert user session");

    // Taking the session removes it and returns its contents.
    let session = admin_portal_session::take_user_session(&db, &session_id)
        .await
        .unwrap()
        .expect("session should be present");
    assert_eq!(session.id_token, "id_token");
    assert_eq!(session.display_name, "Jane Doe");
    assert_eq!(session.roles, vec!["privilege_admin".to_string()]);
    assert_eq!(session.expires_at, expires_at);

    let result = admin_portal_session::take_user_session(&db, &session_id).await.unwrap();
    assert!(result.is_none(), "session should have been removed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_user_session_touch_and_take() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let session_id = random_string(32);
    let now = now();

    admin_portal_session::insert_user_session(
        &db,
        session_id.clone(),
        AdminPortalUserSession {
            display_name: "Jane Doe".to_string(),
            roles: vec!["privilege_admin".to_string()],
            id_token: "id_token".to_string(),
            expires_at: now + TimeDelta::minutes(15),
        },
    )
    .await
    .expect("should insert user session");

    // Extending a valid session should succeed and return the updated session.
    let new_expires_at = now + TimeDelta::minutes(30);
    let session = admin_portal_session::touch_user_session(&db, &session_id, now, new_expires_at)
        .await
        .unwrap()
        .expect("session should be present and not expired");
    assert_eq!(session.display_name, "Jane Doe");
    assert_eq!(session.roles, vec!["privilege_admin".to_string()]);
    assert_eq!(session.expires_at, new_expires_at);

    // Extending an already expired session should not find it.
    let result =
        admin_portal_session::touch_user_session(&db, &session_id, new_expires_at + TimeDelta::minutes(1), now)
            .await
            .unwrap();
    assert!(result.is_none());

    // Taking the session removes it and returns its contents.
    let session = admin_portal_session::take_user_session(&db, &session_id)
        .await
        .unwrap()
        .expect("session should be present");
    assert_eq!(session.id_token, "id_token");

    let result = admin_portal_session::take_user_session(&db, &session_id).await.unwrap();
    assert!(result.is_none(), "session should have been removed");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_cleanup_expired_user_sessions() {
    let db_setup = DbSetup::create().await;
    let db = db_from_setup(&db_setup).await;

    let expired_id = random_string(32);
    let active_id = random_string(32);
    let now = now();

    admin_portal_session::insert_user_session(
        &db,
        expired_id.clone(),
        AdminPortalUserSession {
            display_name: "Expired".to_string(),
            roles: vec![],
            id_token: "id_token".to_string(),
            expires_at: now - TimeDelta::minutes(1),
        },
    )
    .await
    .unwrap();
    admin_portal_session::insert_user_session(
        &db,
        active_id.clone(),
        AdminPortalUserSession {
            display_name: "Active".to_string(),
            roles: vec![],
            id_token: "id_token".to_string(),
            expires_at: now + TimeDelta::minutes(15),
        },
    )
    .await
    .unwrap();

    admin_portal_session::cleanup_expired_user_sessions(&db, now)
        .await
        .expect("should clean up expired user sessions");

    assert!(
        admin_portal_session::take_user_session(&db, &expired_id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        admin_portal_session::take_user_session(&db, &active_id)
            .await
            .unwrap()
            .is_some()
    );
}
