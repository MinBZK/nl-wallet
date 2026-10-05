#![cfg(feature = "db_test")]

use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use db_test::DbSetup;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTaskCreate;
use wallet_provider_domain::model::admin_task::AdminTaskReview;
use wallet_provider_domain::model::admin_task::ReviewVerdict;
use wallet_provider_domain::model::admin_task::TaskListView;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::repository::AdminTaskRepository;
use wallet_provider_persistence::repositories::Repositories;
use wallet_provider_persistence::test::db_from_setup;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn task_lifecycle_and_lists() {
    let setup = DbSetup::create().await;
    let db = db_from_setup(&setup).await;
    let repo = Repositories::from(db);
    let now = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).unwrap();
    let create = |owner: &str, offset| AdminTaskCreate {
        id: Uuid::new_v4(),
        created_at: now + TimeDelta::seconds(offset),
        created_by_user_id: owner.to_owned(),
        created_by_display_name: format!("Name {owner}"),
        task_type: TaskType::BlockUser,
        task_parameters: sea_orm::JsonValue::from("opaque parameters"),
        task_reason: "reason".to_owned(),
    };
    assert!(repo.find_task_by_id(Uuid::new_v4()).await.unwrap().is_none());
    let later = repo.insert_task(create("alice", 2)).await.unwrap();
    let first = repo.insert_task(create("alice", 0)).await.unwrap();
    let other = repo.insert_task(create("bob", 1)).await.unwrap();
    assert_eq!(first.status, TaskStatus::Created);
    assert_eq!(first.reviewed_at, None);
    assert_eq!(first.executed_at, None);
    assert_eq!(first.execution_result, None);
    assert_eq!(first.execution_result_details, None);
    assert_eq!(repo.find_task_by_id(first.id).await.unwrap(), Some(first.clone()));
    assert!(
        repo.insert_task(AdminTaskCreate {
            id: first.id,
            ..create("alice", 0)
        })
        .await
        .is_err()
    );
    assert_eq!(
        repo.list_tasks(TaskListView::OpenTasks {
            user_id: "alice".into()
        })
        .await
        .unwrap(),
        vec![first.clone(), later.clone()]
    );
    assert_eq!(
        repo.list_tasks(TaskListView::Reviewable {
            user_id: "alice".into()
        })
        .await
        .unwrap(),
        vec![other.clone()]
    );
    assert!(repo.withdraw_task(first.id, "bob", now).await.unwrap().is_none());
    assert!(
        repo.withdraw_task(Uuid::new_v4(), "alice", now)
            .await
            .unwrap()
            .is_none()
    );
    let withdrawn = repo.withdraw_task(first.id, "alice", now).await.unwrap().unwrap();
    assert_eq!(withdrawn.status, TaskStatus::Withdrawn);
    assert_eq!(withdrawn.reviewed_at, Some(now));
    assert!(repo.withdraw_task(first.id, "alice", now).await.unwrap().is_none());
    let review = |id, verdict| AdminTaskReview {
        id,
        reviewed_at: now,
        reviewed_by_user_id: "alice".into(),
        reviewed_by_display_name: "Alice".into(),
        review_comment: "review".into(),
        review_verdict: verdict,
    };
    assert!(
        repo.review_task(review(first.id, ReviewVerdict::Approve))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo.review_task(review(Uuid::new_v4(), ReviewVerdict::Approve))
            .await
            .unwrap()
            .is_none()
    );
    let (approved, rejected) = tokio::join!(
        repo.review_task(review(other.id, ReviewVerdict::Approve)),
        repo.review_task(review(other.id, ReviewVerdict::Reject))
    );
    let results = [approved.unwrap(), rejected.unwrap()];
    assert_eq!(results.iter().filter(|result| result.is_some()).count(), 1);
    let reviewed = results.into_iter().flatten().next().unwrap();
    assert_eq!(reviewed.reviewed_by_user_id.as_deref(), Some("alice"));
    assert_eq!(reviewed.reviewed_by_display_name.as_deref(), Some("Alice"));
    assert_eq!(reviewed.review_comment.as_deref(), Some("review"));
    assert!(repo.withdraw_task(other.id, "bob", now).await.unwrap().is_none());
    let rejected = repo
        .review_task(review(later.id, ReviewVerdict::Reject))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(rejected.status, TaskStatus::Rejected);
    assert_eq!(
        repo.list_tasks(TaskListView::History {
            user_id: "alice".into()
        })
        .await
        .unwrap(),
        vec![withdrawn, reviewed.clone(), rejected]
    );
    assert_eq!(
        repo.list_tasks(TaskListView::History { user_id: "bob".into() })
            .await
            .unwrap(),
        vec![reviewed]
    );
    assert!(
        repo.list_tasks(TaskListView::OpenTasks {
            user_id: "alice".into()
        })
        .await
        .unwrap()
        .is_empty()
    );
    assert!(
        repo.list_tasks(TaskListView::Reviewable {
            user_id: "alice".into()
        })
        .await
        .unwrap()
        .is_empty()
    );
}
