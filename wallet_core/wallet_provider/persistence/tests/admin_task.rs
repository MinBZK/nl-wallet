use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use db_test::DbSetup;
use serde_json::json;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTask;
use wallet_provider_domain::model::admin_task::AdminTaskCreate;
use wallet_provider_domain::model::admin_task::AdminTaskReview;
use wallet_provider_domain::model::admin_task::ReviewVerdict;
use wallet_provider_domain::model::admin_task::TaskListView;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_persistence::admin_task;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::test::db_from_setup;

const REVIEW_COMMENT: &str = "review comment";

/// Returns a fixed instant, offset by the given number of minutes. A fixed instant is used in place of `Utc::now()`,
/// because Postgres `timestamptz` only stores microsecond precision, so a real "now" (which carries nanoseconds) would
/// not compare equal to what gets read back from the database.
fn at(minutes: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).expect("timestamp should be in range") + TimeDelta::minutes(minutes)
}

/// An admin portal user. Every user gets a unique id, so tests sharing a database do not see each other's tasks.
struct User {
    id: String,
    display_name: String,
}

impl User {
    fn new(name: &str) -> Self {
        Self {
            id: format!("{name}-{}", Uuid::new_v4()),
            display_name: format!("{name} Display Name"),
        }
    }
}

fn task_create(creator: &User, created_at: DateTime<Utc>) -> AdminTaskCreate {
    AdminTaskCreate {
        id: Uuid::new_v4(),
        created_at,
        created_by_user_id: creator.id.clone(),
        created_by_display_name: creator.display_name.clone(),
        task_type: TaskType::RevokeWallet,
        task_parameters: json!({ "wallet_id": "some-wallet" }),
        task_reason: "because reasons".to_string(),
    }
}

/// Wraps the database, so tests read as a sequence of task operations.
struct TestDatabase {
    _db_setup: DbSetup, // retain the advisory locks
    db: Db,
}

impl TestDatabase {
    async fn new() -> Self {
        let db_setup = DbSetup::create().await;
        let db = db_from_setup(&db_setup).await;
        Self {
            _db_setup: db_setup,
            db,
        }
    }

    async fn insert(&self, task: AdminTaskCreate) -> Result<AdminTask, PersistenceError> {
        admin_task::insert_task(&self.db, task).await
    }

    async fn create_task(&self, creator: &User, created_at: DateTime<Utc>) -> Result<AdminTask, PersistenceError> {
        self.insert(task_create(creator, created_at)).await
    }

    async fn find(&self, id: Uuid) -> Result<Option<AdminTask>, PersistenceError> {
        admin_task::find_task_by_id(&self.db, id).await
    }

    async fn withdraw(
        &self,
        id: Uuid,
        user: &User,
        withdrawn_at: DateTime<Utc>,
    ) -> Result<Option<AdminTask>, PersistenceError> {
        admin_task::withdraw_task(&self.db, id, &user.id, withdrawn_at).await
    }

    async fn review(
        &self,
        id: Uuid,
        reviewer: &User,
        review_verdict: ReviewVerdict,
        reviewed_at: DateTime<Utc>,
    ) -> Result<Option<AdminTask>, PersistenceError> {
        let review = AdminTaskReview {
            id,
            reviewed_at,
            reviewed_by_user_id: reviewer.id.clone(),
            reviewed_by_display_name: reviewer.display_name.clone(),
            review_comment: REVIEW_COMMENT.to_string(),
            review_verdict,
        };
        admin_task::review_task(&self.db, review).await
    }

    async fn list(&self, view: TaskListView) -> Result<Vec<AdminTask>, PersistenceError> {
        admin_task::list_tasks(&self.db, view).await
    }

    async fn open_tasks(&self, user: &User) -> Result<Vec<AdminTask>, PersistenceError> {
        self.list(TaskListView::OpenTasks {
            user_id: user.id.clone(),
        })
        .await
    }

    async fn history(&self, user: &User) -> Result<Vec<AdminTask>, PersistenceError> {
        self.list(TaskListView::History {
            user_id: user.id.clone(),
        })
        .await
    }

    async fn reviewable(&self, user: &User) -> Result<Vec<AdminTask>, PersistenceError> {
        self.list(TaskListView::Reviewable {
            user_id: user.id.clone(),
        })
        .await
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_insert_and_find_task_by_id() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let create = task_create(&alice, at(0));

    assert_eq!(db.find(create.id).await.expect("should query task"), None);

    let inserted = db.insert(create.clone()).await.expect("should insert task");

    let expected = AdminTask {
        id: create.id,
        created_at: create.created_at,
        created_by_user_id: create.created_by_user_id,
        created_by_display_name: create.created_by_display_name,
        task_type: create.task_type,
        task_parameters: create.task_parameters,
        task_reason: create.task_reason,
        status: TaskStatus::Created,
        reviewed_at: None,
        reviewed_by_user_id: None,
        reviewed_by_display_name: None,
        review_comment: None,
        executed_at: None,
        execution_result: None,
        execution_result_details: None,
    };
    assert_eq!(inserted, expected);
    assert_eq!(db.find(inserted.id).await.expect("should query task"), Some(expected));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_insert_fails_for_duplicate_id() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");

    let duplicate = AdminTaskCreate {
        id: task.id,
        ..task_create(&alice, at(1))
    };

    assert!(db.insert(duplicate).await.is_err());
    assert_eq!(
        db.find(task.id).await.expect("should query task"),
        Some(task),
        "original task must be unchanged"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_succeeds_for_creator_of_created_task() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");

    let withdrawn = db
        .withdraw(task.id, &alice, at(1))
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");

    assert_eq!(withdrawn.status, TaskStatus::Withdrawn);
    assert_eq!(withdrawn.reviewed_at, Some(at(1)));
    assert_eq!(withdrawn.reviewed_by_user_id, None);
    assert_eq!(db.find(task.id).await.expect("should query task"), Some(withdrawn));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_fails_for_unknown_task() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");

    assert_eq!(
        db.withdraw(Uuid::new_v4(), &alice, at(0))
            .await
            .expect("should execute withdrawal"),
        None
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_fails_for_other_user() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");

    assert_eq!(
        db.withdraw(task.id, &bob, at(1))
            .await
            .expect("should execute withdrawal"),
        None
    );
    assert_eq!(
        db.find(task.id).await.expect("should query task"),
        Some(task),
        "task must be unchanged"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_fails_when_already_withdrawn() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");
    let withdrawn = db
        .withdraw(task.id, &alice, at(1))
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");

    assert_eq!(
        db.withdraw(task.id, &alice, at(2))
            .await
            .expect("should execute withdrawal"),
        None
    );
    assert_eq!(
        db.find(task.id).await.expect("should query task"),
        Some(withdrawn),
        "first withdrawal must stick"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_fails_when_already_reviewed() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");
    let approved = db
        .review(task.id, &bob, ReviewVerdict::Approve, at(1))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");

    assert_eq!(
        db.withdraw(task.id, &alice, at(2))
            .await
            .expect("should execute withdrawal"),
        None
    );
    assert_eq!(
        db.find(task.id).await.expect("should query task"),
        Some(approved),
        "review must stick"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_approve_sets_status_and_reviewer_fields() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");

    let approved = db
        .review(task.id, &bob, ReviewVerdict::Approve, at(1))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");

    assert_eq!(
        approved,
        AdminTask {
            status: TaskStatus::Approved,
            reviewed_at: Some(at(1)),
            reviewed_by_user_id: Some(bob.id),
            reviewed_by_display_name: Some(bob.display_name),
            review_comment: Some(REVIEW_COMMENT.to_string()),
            ..task
        }
    );
    assert_eq!(db.find(task.id).await.expect("should query task"), Some(approved));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_reject_sets_rejected_status() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");

    let rejected = db
        .review(task.id, &bob, ReviewVerdict::Reject, at(1))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");

    assert_eq!(rejected.status, TaskStatus::Rejected);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_fails_for_unknown_task() {
    let db = TestDatabase::new().await;
    let bob = User::new("bob");

    assert_eq!(
        db.review(Uuid::new_v4(), &bob, ReviewVerdict::Approve, at(0))
            .await
            .expect("should execute review"),
        None
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_fails_when_already_reviewed() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let carol = User::new("carol");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");
    let approved = db
        .review(task.id, &bob, ReviewVerdict::Approve, at(1))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");

    assert_eq!(
        db.review(task.id, &carol, ReviewVerdict::Reject, at(2))
            .await
            .expect("should execute review"),
        None
    );
    assert_eq!(
        db.find(task.id).await.expect("should query task"),
        Some(approved),
        "first review must stick"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_fails_when_withdrawn() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");
    let withdrawn = db
        .withdraw(task.id, &alice, at(1))
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");

    assert_eq!(
        db.review(task.id, &bob, ReviewVerdict::Approve, at(2))
            .await
            .expect("should execute review"),
        None
    );
    assert_eq!(
        db.find(task.id).await.expect("should query task"),
        Some(withdrawn),
        "withdrawal must stick"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_concurrent_reviews_only_one_succeeds() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let carol = User::new("carol");
    let task = db.create_task(&alice, at(0)).await.expect("should create task");

    let (approved, rejected) = tokio::join!(
        db.review(task.id, &bob, ReviewVerdict::Approve, at(1)),
        db.review(task.id, &carol, ReviewVerdict::Reject, at(1)),
    );

    let winners = [
        approved.expect("should execute approval"),
        rejected.expect("should execute rejection"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let [winner] = winners.as_slice() else {
        panic!("exactly one review must succeed, got {}", winners.len());
    };
    assert_eq!(
        db.find(task.id).await.expect("should query task").as_ref(),
        Some(winner)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_open_tasks_returns_own_created_tasks_oldest_first() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");

    let newer = db.create_task(&alice, at(1)).await.expect("should create task");
    let older = db.create_task(&alice, at(0)).await.expect("should create task");
    let _of_other_user = db.create_task(&bob, at(0)).await.expect("should create task");
    let withdrawn = db.create_task(&alice, at(0)).await.expect("should create task");
    db.withdraw(withdrawn.id, &alice, at(2))
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");

    assert_eq!(
        db.open_tasks(&alice).await.expect("should list open tasks"),
        vec![older, newer]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_history_returns_finished_tasks_created_or_reviewed_by_user_oldest_first() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let carol = User::new("carol");

    let reviewed_by_alice = db.create_task(&carol, at(2)).await.expect("should create task");
    let reviewed_by_alice = db
        .review(reviewed_by_alice.id, &alice, ReviewVerdict::Reject, at(3))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");

    let created_by_alice_and_approved = db.create_task(&alice, at(1)).await.expect("should create task");
    let created_by_alice_and_approved = db
        .review(created_by_alice_and_approved.id, &bob, ReviewVerdict::Approve, at(3))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");

    let withdrawn_by_alice = db.create_task(&alice, at(0)).await.expect("should create task");
    let withdrawn_by_alice = db
        .withdraw(withdrawn_by_alice.id, &alice, at(3))
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");

    let _still_open = db.create_task(&alice, at(0)).await.expect("should create task");
    let reviewed_by_other_user = db.create_task(&carol, at(0)).await.expect("should create task");
    db.review(reviewed_by_other_user.id, &bob, ReviewVerdict::Approve, at(3))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");

    assert_eq!(
        db.history(&alice).await.expect("should list task history"),
        vec![withdrawn_by_alice, created_by_alice_and_approved, reviewed_by_alice]
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_reviewable_returns_created_tasks_of_other_users_oldest_first() {
    let db = TestDatabase::new().await;
    let alice = User::new("alice");
    let bob = User::new("bob");
    let carol = User::new("carol");

    let newer = db.create_task(&carol, at(1)).await.expect("should create task");
    let older = db.create_task(&bob, at(0)).await.expect("should create task");
    let _own = db.create_task(&alice, at(0)).await.expect("should create task");
    let already_reviewed = db.create_task(&bob, at(0)).await.expect("should create task");
    db.review(already_reviewed.id, &carol, ReviewVerdict::Approve, at(2))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");
    let withdrawn = db.create_task(&bob, at(0)).await.expect("should create task");
    db.withdraw(withdrawn.id, &bob, at(2))
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");

    // Reviewable tasks are not scoped to a creator, so ignore tasks other tests left in the shared database.
    let reviewable = db
        .reviewable(&alice)
        .await
        .expect("should list reviewable tasks")
        .into_iter()
        .filter(|task| [&bob.id, &carol.id].contains(&&task.created_by_user_id))
        .collect::<Vec<_>>();
    assert_eq!(reviewable, vec![older, newer]);
}
