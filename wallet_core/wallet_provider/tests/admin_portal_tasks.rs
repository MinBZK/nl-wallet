//! Database integration tests for the Admin Portal's task endpoints (`create`/`withdraw`/`review`/`list`).
//!
//! These exercise the real HTTP router (`admin_portal::router`) against a real, migrated Postgres database for
//! both the main schema and the audit log schema, wired up exactly like production (see `router_state.rs`). Only
//! the happy flow of each operation is covered here; the not-found/forbidden/invalid-state/conflict paths are
//! already covered by the HTTP-layer integration tests in `src/admin_portal.rs` and the
//! persistence-layer DB tests in `wallet_provider_persistence/tests/admin_task.rs`.
//! Although E2E browser tests also cover these flows, these tests provide shorter iteration times, but don't include
//! the frontend.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::body::to_bytes;
use axum::http::Request;
use axum::http::StatusCode;
use axum::http::header;
use chrono::Utc;
use serde::de::DeserializeOwned;
use serde_json::json;
use tower::ServiceExt;
use utils::vec_nonempty;
use uuid::Uuid;
use wallet_provider::admin_portal;
use wallet_provider::admin_portal::CheckWalletRequest;
use wallet_provider::admin_portal::CheckWalletResponse;
use wallet_provider::admin_portal::CreateTaskRequest;
use wallet_provider::admin_portal::ReviewTaskRequest;
use wallet_provider::admin_portal::TaskResponse;
use wallet_provider::admin_portal::WithdrawTaskRequest;
use wallet_provider::test_support::TestState;
use wallet_provider::test_support::setup_state;
use wallet_provider_domain::model::admin_portal_session::AdminPortalUserSession;
use wallet_provider_domain::model::admin_task::CreateAdminTask;
use wallet_provider_domain::model::admin_task::ReviewVerdict;
use wallet_provider_domain::model::admin_task::RevokeWalletParameters;
use wallet_provider_domain::model::admin_task::TaskPayload;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::model::admin_task::WithdrawAdminTask;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::repository::AdminPortalRepository;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_persistence::test::WalletDeviceVendor;
use wallet_provider_persistence::test::create_wallet_user_with_random_keys;
use wallet_provider_service::admin_portal::wallet_lookup::WalletRevocationState;

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

async fn test_admin_portal_app() -> (Router, Repositories, TestState) {
    let setup = setup_state().await;
    (
        admin_portal::router(Arc::clone(&setup)),
        setup.repositories.clone(),
        setup,
    )
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

fn revoke_wallet_payload() -> TaskPayload {
    TaskPayload::RevokeWallet(RevokeWalletParameters {
        wallet_ids: vec_nonempty!["wallet-1".to_string().into()].into(),
    })
}

/// Inserts a wallet user with a unique wallet id and returns that wallet id.
async fn seed_wallet(repositories: &Repositories) -> WalletId {
    let wallet_id = WalletId::from(format!("wallet-{}", Uuid::new_v4()));
    let transaction = repositories.begin_transaction().await.unwrap();
    create_wallet_user_with_random_keys(&transaction, WalletDeviceVendor::Apple, wallet_id.clone()).await;
    transaction.commit().await.unwrap();
    wallet_id
}

async fn seed_task(repositories: &Repositories, creator: &TestUser) -> Uuid {
    seed_task_with_payload(repositories, creator, revoke_wallet_payload()).await
}

async fn seed_task_with_payload(repositories: &Repositories, creator: &TestUser, payload: TaskPayload) -> Uuid {
    let task = repositories
        .insert_task(CreateAdminTask {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            created_by_user_id: creator.id.clone(),
            created_by_display_name: creator.display_name.clone(),
            payload,
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

#[rstest::rstest]
#[case::withdraw(None, None)]
#[case::approve(Some(ReviewVerdict::Approve), Some(ReviewVerdict::Approve))]
#[case::reject(Some(ReviewVerdict::Reject), Some(ReviewVerdict::Reject))]
#[case::opposite_verdicts(Some(ReviewVerdict::Approve), Some(ReviewVerdict::Reject))]
#[case::withdraw_vs_approve(None, Some(ReviewVerdict::Approve))]
#[case::withdraw_vs_reject(None, Some(ReviewVerdict::Reject))]
#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_concurrent_task_mutations(
    #[case] first_verdict: Option<ReviewVerdict>,
    #[case] second_verdict: Option<ReviewVerdict>,
) {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let bob = TestUser::new("bob");
    let creator_cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let reviewer_cookie = seed_session(&repositories, &bob, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice).await;

    let request = |verdict: Option<ReviewVerdict>| {
        let (action, cookie, body) = match verdict {
            None => (
                "withdraw",
                &creator_cookie,
                Body::from(
                    serde_json::to_string(&WithdrawTaskRequest {
                        withdrawal_comment: "concurrent withdrawal".to_string(),
                    })
                    .unwrap(),
                ),
            ),
            Some(verdict) => (
                "review",
                &reviewer_cookie,
                Body::from(
                    serde_json::to_string(&ReviewTaskRequest {
                        verdict,
                        review_comment: "concurrent review".to_string(),
                    })
                    .unwrap(),
                ),
            ),
        };
        Request::builder()
            .method("POST")
            .uri(format!("/tasks/{task_id}/{action}"))
            .header(header::COOKIE, cookie)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .unwrap()
    };

    // Both endpoints must wait for the same database row lock before checking its state.
    let transaction = repositories.begin_transaction().await.unwrap();
    repositories
        .find_task_by_id_for_update(&transaction, task_id)
        .await
        .unwrap()
        .unwrap();
    let first = app.clone().oneshot(request(first_verdict));
    let second = app.oneshot(request(second_verdict));
    tokio::pin!(first, second);
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut first)
            .await
            .is_err()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut second)
            .await
            .is_err()
    );
    transaction.commit().await.unwrap();

    let (first, second) =
        tokio::time::timeout(Duration::from_secs(10), async { tokio::join!(&mut first, &mut second) })
            .await
            .expect("both requests should finish after the lock is released");
    let first = first.unwrap();
    let second = second.unwrap();
    if first_verdict == second_verdict {
        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(second.status(), StatusCode::OK);
        assert_eq!(
            json_body::<serde_json::Value>(first).await,
            json_body::<serde_json::Value>(second).await
        );
    } else {
        assert!(matches!(
            (first.status(), second.status()),
            (StatusCode::OK, StatusCode::CONFLICT) | (StatusCode::CONFLICT, StatusCode::OK)
        ));
    }
}

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_check_wallet_succeeds() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let wallet_id = seed_wallet(&repositories).await;

    let request = Request::builder()
        .method("POST")
        .uri("/wallets/check")
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&CheckWalletRequest { wallet_id }).unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body: CheckWalletResponse = json_body(response).await;
    assert_eq!(body.status, WalletRevocationState::Revokable);
}

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_check_wallet_bad_request() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;

    let request = Request::builder()
        .method("POST")
        .uri("/wallets/check")
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "walletId": 3 }).to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_task_succeeds() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let wallet_id = seed_wallet(&repositories).await;
    let payload = TaskPayload::RevokeWallet(RevokeWalletParameters {
        wallet_ids: vec_nonempty![wallet_id].into(),
    });

    let request = Request::builder()
        .method("POST")
        .uri("/tasks")
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&CreateTaskRequest {
                payload: payload.clone(),
                task_reason: "integration test".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let task: TaskResponse = json_body(response).await;
    assert_eq!(task.status, TaskStatus::Created);
    assert_eq!(task.payload, payload);
    assert_eq!(task.created_by, alice.display_name);
}

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_task_succeeds() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice).await;

    let request = Request::builder()
        .method("POST")
        .uri(format!("/tasks/{task_id}/withdraw"))
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&WithdrawTaskRequest {
                withdrawal_comment: "no longer needed".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let task: TaskResponse = json_body(response).await;
    assert_eq!(task.task_id, task_id);
    assert_eq!(task.status, TaskStatus::Withdrawn);
    assert_eq!(task.withdrawal_comment, Some("no longer needed".to_string()));
}

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_task_approve_succeeds() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let bob = TestUser::new("bob");
    let cookie = seed_session(&repositories, &bob, &[TaskType::RevokeWallet]).await;
    let wallet_id = seed_wallet(&repositories).await;
    let payload = TaskPayload::RevokeWallet(RevokeWalletParameters {
        wallet_ids: vec_nonempty![wallet_id].into(),
    });
    let task_id = seed_task_with_payload(&repositories, &alice, payload).await;

    let response = app
        .oneshot(review_request(task_id, &cookie, ReviewVerdict::Approve))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let task: TaskResponse = json_body(response).await;
    assert_eq!(task.task_id, task_id);
    assert_eq!(task.status, TaskStatus::Approved);
    assert_eq!(task.reviewed_by, Some(bob.display_name));
    assert_eq!(task.review_comment, Some("looks good".to_string()));
    assert!(task.executed_at.is_some());
    assert_eq!(task.execution_successful, Some(true));
    assert_eq!(task.execution_details, Some("revoked 1 wallet(s)".to_string()));
}

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_task_approve_stores_failed_execution_for_unknown_wallet() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let bob = TestUser::new("bob");
    let cookie = seed_session(&repositories, &bob, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice).await;

    let response = app
        .oneshot(review_request(task_id, &cookie, ReviewVerdict::Approve))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let task: TaskResponse = json_body(response).await;
    assert_eq!(task.status, TaskStatus::Approved);
    assert!(task.executed_at.is_some());
    assert_eq!(task.execution_successful, Some(false));
    assert!(task.execution_details.unwrap().contains("wallet ID not found"));
}

fn review_request(task_id: Uuid, cookie: &str, verdict: ReviewVerdict) -> Request<Body> {
    let review_comment = match verdict {
        ReviewVerdict::Approve => "looks good",
        ReviewVerdict::Reject => "not today",
    };
    Request::builder()
        .method("POST")
        .uri(format!("/tasks/{task_id}/review"))
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_string(&ReviewTaskRequest {
                verdict,
                review_comment: review_comment.to_string(),
            })
            .unwrap(),
        ))
        .unwrap()
}

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_tasks_open_succeeds() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice).await;

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

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_tasks_history_succeeds() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let cookie = seed_session(&repositories, &alice, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice).await;
    let transaction = repositories.begin_transaction().await.unwrap();
    repositories
        .withdraw_task(
            &transaction,
            WithdrawAdminTask {
                task_id,
                withdrawal_comment: "no longer needed".to_string(),
                withdrawn_at: Utc::now(),
            },
        )
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");
    transaction.commit().await.unwrap();

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

#[serial_test::serial(hsm)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_list_tasks_reviewable_succeeds() {
    let (app, repositories, _db_setup) = test_admin_portal_app().await;
    let alice = TestUser::new("alice");
    let bob = TestUser::new("bob");
    let cookie = seed_session(&repositories, &bob, &[TaskType::RevokeWallet]).await;
    let task_id = seed_task(&repositories, &alice).await;

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
