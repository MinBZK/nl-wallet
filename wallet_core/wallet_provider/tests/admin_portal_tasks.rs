//! Database integration tests for the Admin Portal's task endpoints (`create`/`withdraw`/`review`/`list`).
//!
//! These exercise the real HTTP router (`admin_portal::router`) against a real, migrated Postgres database for
//! both the main schema and the audit log schema, wired up exactly like production (see `router_state.rs`). Only
//! the happy flow of each operation is covered here; the not-found/forbidden/invalid-state/conflict paths are
//! already covered by the HTTP-layer unit tests in `src/admin_portal.rs` (against an in-memory repository) and the
//! persistence-layer DB tests in `wallet_provider_persistence/tests/admin_task.rs`.

use std::sync::Arc;
use std::time::Duration;

use audit_log::model::DbAuditLog;
use audit_log::repository::PostgresAuditLogRepository;
use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::Request;
use axum::http::StatusCode;
use axum::http::header;
use chrono::Utc;
use db_test::DbSetup;
use http_utils::reqwest::test::get_test_trust_anchor;
use serde::de::DeserializeOwned;
use serde_json::json;
use tower::ServiceExt;
use utils::generator::TimeGenerator;
use utils::vec_nonempty;
use uuid::Uuid;
use wallet_provider::admin_portal;
use wallet_provider::admin_portal::AdminPortalState;
use wallet_provider::admin_portal::CreateTaskRequest;
use wallet_provider::admin_portal::ReviewTaskRequest;
use wallet_provider::admin_portal::TaskResponse;
use wallet_provider::router_state::UuidV7Generator;
use wallet_provider::settings::AdminPortalSettings;
use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
use wallet_provider_domain::model::admin_task::AdminTaskCreate;
use wallet_provider_domain::model::admin_task::ReviewVerdict;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::repository::AdminPortalSessionRepository;
use wallet_provider_domain::repository::AdminTaskRepository;
use wallet_provider_persistence::database::ConnectionOptions;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_persistence::test::db_from_setup;

/// An admin portal user. Every user gets a unique id, so tests sharing a database do not see each other's data.
struct TestUser {
    id: String,
    display_name: String,
}

impl TestUser {
    fn new(name: &str) -> Self {
        Self {
            id: format!("{name}-{}", Uuid::new_v4()),
            display_name: format!("{name} Display Name"),
        }
    }
}

/// Sets up a real, migrated database (main schema + audit log schema) and returns the resulting router, along with
/// direct repository access for seeding data and `DbSetup` (which must stay alive for the duration of the test).
async fn test_app() -> (Router, Repositories, DbSetup) {
    let db_setup = DbSetup::create().await;
    let repositories = Repositories::from(db_from_setup(&db_setup).await);

    let connection_options = ConnectionOptions {
        acquire_timeout: Duration::from_secs(5),
        connect_timeout: Duration::from_secs(1),
        max_connections: 5,
    };
    let audit_db_connection = Db::new_connection(db_setup.audit_log_url(), connection_options)
        .await
        .expect("should connect to audit log database");

    let state = Arc::new(
        AdminPortalState::try_new(
            AdminPortalSettings {
                // .invalid is reserved by the IETF and will never resolve
                keycloak_url: "https://keycloak.invalid".parse().unwrap(),
                keycloak_realm: "test-realm".to_string(),
                keycloak_client_id: "test-client".to_string(),
                public_url: "https://portal.example.org/".parse().unwrap(),
                keycloak_trust_anchors: vec_nonempty![get_test_trust_anchor()],
                session_ttl: Duration::from_secs(900),
                login_attempt_ttl: Duration::from_secs(600),
            },
            repositories.clone(),
            DbAuditLog {
                repository: PostgresAuditLogRepository {
                    db_connection: audit_db_connection,
                },
                time_generator: TimeGenerator,
                uuid_generator: UuidV7Generator,
            },
        )
        .expect("should construct admin portal state"),
    );

    (admin_portal::router(state), repositories, db_setup)
}

/// Inserts a session for `user`, authorized for `privileges`, and returns the `sid` cookie header value.
async fn seed_session(repositories: &Repositories, user: &TestUser, privileges: &[TaskType]) -> String {
    let session_id = Uuid::new_v4().to_string();
    repositories
        .insert_user_session(
            session_id.clone(),
            AdminPortalUserSession {
                user_id: user.id.clone(),
                display_name: user.display_name.clone(),
                roles: privileges.iter().map(|p| format!("privilege_{p}")).collect(),
                id_token: "the-id-token".to_string(),
                expires_at: Utc::now() + chrono::TimeDelta::minutes(5),
            },
        )
        .await
        .expect("should insert session");

    format!("sid={session_id}")
}

async fn seed_task(repositories: &Repositories, creator: &TestUser, task_type: TaskType) -> Uuid {
    let task = repositories
        .insert_task(AdminTaskCreate {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            created_by_user_id: creator.id.clone(),
            created_by_display_name: creator.display_name.clone(),
            task_type,
            task_parameters: json!({}),
            task_reason: "because reasons".to_string(),
        })
        .await
        .expect("should insert task");

    task.id
}

async fn json_body<T: DeserializeOwned>(response: axum::response::Response) -> T {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_task_succeeds() {
    let (app, repositories, _db_setup) = test_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;

    let request = Request::builder()
        .method("POST")
        .uri("/tasks")
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&CreateTaskRequest {
                task_type: TaskType::RevokeWallet.to_string(),
                task_parameters: serde_json::Map::new(),
                task_reason: "integration test".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let task: TaskResponse = json_body(response).await;
    assert_eq!(task.status, TaskStatus::Created);
    assert_eq!(task.task_type, TaskType::RevokeWallet);
    assert_eq!(task.created_by, alice.display_name);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_task_succeeds() {
    let (app, repositories, _db_setup) = test_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice, TaskType::RevokeWallet).await;

    let request = Request::builder()
        .method("POST")
        .uri(format!("/tasks/{task_id}/withdraw"))
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let task: TaskResponse = json_body(response).await;
    assert_eq!(task.task_id, task_id);
    assert_eq!(task.status, TaskStatus::Withdrawn);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_task_approve_succeeds() {
    let (app, repositories, _db_setup) = test_app().await;
    let alice = TestUser::new("alice");
    let bob = TestUser::new("bob");
    let cookie = seed_session(&repositories, &bob, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice, TaskType::RevokeWallet).await;

    let request = Request::builder()
        .method("POST")
        .uri(format!("/tasks/{task_id}/review"))
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&ReviewTaskRequest {
                verdict: ReviewVerdict::Approve,
                review_comment: "looks good".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let task: TaskResponse = json_body(response).await;
    assert_eq!(task.task_id, task_id);
    assert_eq!(task.status, TaskStatus::Approved);
    assert_eq!(task.reviewed_by, Some(bob.display_name));
    assert_eq!(task.review_comment, Some("looks good".to_string()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_task_reject_succeeds() {
    let (app, repositories, _db_setup) = test_app().await;
    let alice = TestUser::new("alice");
    let bob = TestUser::new("bob");
    let cookie = seed_session(&repositories, &bob, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice, TaskType::RevokeWallet).await;

    let request = Request::builder()
        .method("POST")
        .uri(format!("/tasks/{task_id}/review"))
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&ReviewTaskRequest {
                verdict: ReviewVerdict::Reject,
                review_comment: "not today".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let task: TaskResponse = json_body(response).await;
    assert_eq!(task.task_id, task_id);
    assert_eq!(task.status, TaskStatus::Rejected);
    assert_eq!(task.reviewed_by, Some(bob.display_name));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_tasks_open_succeeds() {
    let (app, repositories, _db_setup) = test_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice, TaskType::RevokeWallet).await;

    let request = Request::builder()
        .method("GET")
        .uri("/tasks?view=open")
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let tasks: Vec<TaskResponse> = json_body(response).await;
    assert!(tasks.iter().any(|task| task.task_id == task_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_tasks_history_succeeds() {
    let (app, repositories, _db_setup) = test_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice, TaskType::RevokeWallet).await;
    repositories
        .withdraw_task(task_id, &alice.id, Utc::now())
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");

    let request = Request::builder()
        .method("GET")
        .uri("/tasks?view=history")
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let tasks: Vec<TaskResponse> = json_body(response).await;
    assert!(tasks.iter().any(|task| task.task_id == task_id));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_tasks_reviewable_succeeds() {
    let (app, repositories, _db_setup) = test_app().await;
    let alice = TestUser::new("alice");
    let bob = TestUser::new("bob");
    let cookie = seed_session(&repositories, &bob, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice, TaskType::RevokeWallet).await;

    let request = Request::builder()
        .method("GET")
        .uri("/tasks?view=reviewable")
        .header(header::COOKIE, cookie)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let tasks: Vec<TaskResponse> = json_body(response).await;
    assert!(tasks.iter().any(|task| task.task_id == task_id));
}
