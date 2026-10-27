use chrono::DateTime;
use chrono::TimeDelta;
use chrono::Utc;
use db_test::DbName;
use db_test::DbSetup;
use rstest::fixture;
use rstest::rstest;
use uuid::Uuid;
use wallet_provider_domain::model::admin_task::AdminTask;
use wallet_provider_domain::model::admin_task::CreateAdminTask;
use wallet_provider_domain::model::admin_task::ExecuteAdminTask;
use wallet_provider_domain::model::admin_task::ReviewAdminTask;
use wallet_provider_domain::model::admin_task::ReviewVerdict;
use wallet_provider_domain::model::admin_task::RevokeWalletParameters;
use wallet_provider_domain::model::admin_task::TaskListView;
use wallet_provider_domain::model::admin_task::TaskPayload;
use wallet_provider_domain::model::admin_task::TaskStatus;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::model::admin_task::WithdrawAdminTask;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_persistence::admin_task;
use wallet_provider_persistence::database::Db;
use wallet_provider_persistence::test::db_from_setup;

const WITHDRAWAL_COMMENT: &str = "no longer needed";
const REVIEW_COMMENT: &str = "review comment";
const EXECUTION_DETAILS: &str = "execution details";

/// An admin portal user.
struct User {
    id: &'static str,
    display_name: &'static str,
}

const ALICE: User = User {
    id: "alice",
    display_name: "alice Display Name",
};
const BOB: User = User {
    id: "bob",
    display_name: "bob Display Name",
};
const CAROL: User = User {
    id: "carol",
    display_name: "carol Display Name",
};

/// Returns a fixed instant, `minutes` after the base.
fn minutes(minutes: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).expect("timestamp should be in range") + TimeDelta::minutes(minutes)
}

fn task_payload(task_type: TaskType) -> TaskPayload {
    match task_type {
        TaskType::RevokeWallet => TaskPayload::RevokeWallet(RevokeWalletParameters {
            wallet_ids: vec!["some-wallet".to_string().into()].try_into().unwrap(),
        }),
        TaskType::BlockUser => TaskPayload::BlockUser {},
        TaskType::UnblockUser => TaskPayload::UnblockUser {},
        TaskType::RevokeSolution => TaskPayload::RevokeSolution {},
    }
}

fn task_create(creator: &User, created_at: DateTime<Utc>, task_type: TaskType) -> CreateAdminTask {
    CreateAdminTask {
        id: Uuid::new_v4(),
        created_at,
        created_by_user_id: creator.id.to_string(),
        created_by_display_name: creator.display_name.to_string(),
        payload: task_payload(task_type),
        task_reason: "because reasons".to_string(),
    }
}

/// The state a task is brought into before the operation under test.
#[derive(Debug, Clone, Copy)]
enum State {
    Approved,
    Rejected,
    Withdrawn,
    Executed,
}

/// Wraps the database, so tests read as a sequence of task operations.
///
/// The fallible methods return the repository result, for the operation under test. The infallible methods bring a
/// task into a given state, for test setup.
struct TestDatabase {
    _db_setup: DbSetup, // keep the exclusive use of the database
    db: Db,
}

#[fixture]
async fn db() -> TestDatabase {
    TestDatabase::new(DbSetup::create().await).await
}

/// A database with an empty `admin_task` table, for tests that list all tasks.
#[fixture]
async fn clean_db() -> TestDatabase {
    TestDatabase::new(DbSetup::create_clean_only([DbName::WalletProvider]).await).await
}

impl TestDatabase {
    async fn new(db_setup: DbSetup) -> Self {
        let db = db_from_setup(&db_setup).await;
        Self {
            _db_setup: db_setup,
            db,
        }
    }

    async fn insert(&self, task: CreateAdminTask) -> Result<AdminTask, PersistenceError> {
        admin_task::insert_task(&self.db, task).await
    }

    async fn find(&self, id: Uuid) -> Result<Option<AdminTask>, PersistenceError> {
        admin_task::find_task_by_id(&self.db, id).await
    }

    async fn withdraw(&self, id: Uuid, withdrawn_at: DateTime<Utc>) -> Result<Option<AdminTask>, PersistenceError> {
        let withdrawal = WithdrawAdminTask {
            task_id: id,
            withdrawal_comment: WITHDRAWAL_COMMENT.to_string(),
            withdrawn_at,
        };
        admin_task::withdraw_task(&self.db, withdrawal).await
    }

    async fn review(
        &self,
        id: Uuid,
        reviewer: &User,
        review_verdict: ReviewVerdict,
        reviewed_at: DateTime<Utc>,
    ) -> Result<Option<AdminTask>, PersistenceError> {
        let review = ReviewAdminTask {
            task_id: id,
            reviewed_at,
            reviewed_by_user_id: reviewer.id.to_string(),
            reviewed_by_display_name: reviewer.display_name.to_string(),
            review_comment: REVIEW_COMMENT.to_string(),
            review_verdict,
        };
        admin_task::review_task(&self.db, review).await
    }

    async fn execute(
        &self,
        id: Uuid,
        execution_successful: bool,
        executed_at: DateTime<Utc>,
    ) -> Result<Option<AdminTask>, PersistenceError> {
        let execution = ExecuteAdminTask {
            task_id: id,
            executed_at,
            execution_successful,
            execution_details: EXECUTION_DETAILS.to_string(),
        };
        admin_task::store_execution_result(&self.db, execution).await
    }

    async fn list(&self, view: TaskListView) -> Result<Vec<AdminTask>, PersistenceError> {
        admin_task::list_tasks(&self.db, view).await
    }

    async fn open_tasks(&self, user: &User) -> Result<Vec<AdminTask>, PersistenceError> {
        self.list(TaskListView::Open {
            user_id: user.id.to_string(),
        })
        .await
    }

    async fn history(&self, user: &User) -> Result<Vec<AdminTask>, PersistenceError> {
        self.list(TaskListView::History {
            user_id: user.id.to_string(),
        })
        .await
    }

    async fn reviewable(
        &self,
        user: &User,
        authorized_task_types: Vec<TaskType>,
    ) -> Result<Vec<AdminTask>, PersistenceError> {
        self.list(TaskListView::Reviewable {
            user_id: user.id.to_string(),
            authorized_task_types,
        })
        .await
    }

    async fn created_of_type(&self, creator: &User, created_at: DateTime<Utc>, task_type: TaskType) -> AdminTask {
        self.insert(task_create(creator, created_at, task_type))
            .await
            .expect("should create task")
    }

    async fn created(&self, creator: &User, created_at: DateTime<Utc>) -> AdminTask {
        self.created_of_type(creator, created_at, TaskType::RevokeWallet).await
    }

    async fn withdrawn(&self, id: Uuid, withdrawn_at: DateTime<Utc>) -> AdminTask {
        self.withdraw(id, withdrawn_at)
            .await
            .expect("should execute withdrawal")
            .expect("task should be withdrawn")
    }

    async fn reviewed(
        &self,
        id: Uuid,
        reviewer: &User,
        verdict: ReviewVerdict,
        reviewed_at: DateTime<Utc>,
    ) -> AdminTask {
        self.review(id, reviewer, verdict, reviewed_at)
            .await
            .expect("should execute review")
            .expect("task should be reviewed")
    }

    async fn executed(&self, id: Uuid, execution_successful: bool, executed_at: DateTime<Utc>) -> AdminTask {
        self.execute(id, execution_successful, executed_at)
            .await
            .expect("should store execution result")
            .expect("task should be executed")
    }

    /// Creates a task by [`ALICE`] and brings it into `state`, reviewed by [`BOB`] if applicable.
    async fn task_in(&self, state: State) -> AdminTask {
        let task = self.created(&ALICE, minutes(0)).await;
        match state {
            State::Withdrawn => self.withdrawn(task.id, minutes(1)).await,
            State::Approved => self.reviewed(task.id, &BOB, ReviewVerdict::Approve, minutes(1)).await,
            State::Rejected => self.reviewed(task.id, &BOB, ReviewVerdict::Reject, minutes(1)).await,
            State::Executed => {
                self.reviewed(task.id, &BOB, ReviewVerdict::Approve, minutes(1)).await;
                self.executed(task.id, true, minutes(2)).await
            }
        }
    }

    /// Asserts that `task` is stored exactly as given.
    async fn assert_stored(&self, task: &AdminTask, message: &str) {
        assert_eq!(
            self.find(task.id).await.expect("should query task").as_ref(),
            Some(task),
            "{message}"
        );
    }
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_insert_and_find_task_by_id(#[future(awt)] db: TestDatabase) {
    let create = task_create(&ALICE, minutes(0), TaskType::RevokeWallet);

    assert_eq!(db.find(create.id).await.expect("should query task"), None);

    let inserted = db.insert(create.clone()).await.expect("should insert task");

    assert_eq!(inserted, AdminTask::from(create));
    db.assert_stored(&inserted, "inserted task must be stored").await;
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_succeeds_for_creator_of_created_task(#[future(awt)] db: TestDatabase) {
    let task = db.created(&ALICE, minutes(0)).await;

    let withdrawn = db
        .withdraw(task.id, minutes(1))
        .await
        .expect("should execute withdrawal")
        .expect("task should be withdrawn");

    assert_eq!(
        withdrawn,
        AdminTask {
            status: TaskStatus::Withdrawn,
            withdrawal_comment: Some(WITHDRAWAL_COMMENT.to_string()),
            withdrawn_at: Some(minutes(1)),
            ..task
        }
    );
    db.assert_stored(&withdrawn, "withdrawal must be stored").await;
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_fails_for_unknown_task(#[future(awt)] db: TestDatabase) {
    assert_eq!(
        db.withdraw(Uuid::new_v4(), minutes(0))
            .await
            .expect("should execute withdrawal"),
        None
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_fails_when_not_created(
    #[future(awt)] db: TestDatabase,
    #[values(State::Approved, State::Withdrawn)] state: State,
) {
    let task = db.task_in(state).await;

    assert_eq!(
        db.withdraw(task.id, minutes(5))
            .await
            .expect("should execute withdrawal"),
        None
    );
    db.assert_stored(&task, "earlier state must stick").await;
}

#[rstest]
#[case::approve(ReviewVerdict::Approve, TaskStatus::Approved)]
#[case::reject(ReviewVerdict::Reject, TaskStatus::Rejected)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_sets_status_and_reviewer_fields(
    #[future(awt)] db: TestDatabase,
    #[case] verdict: ReviewVerdict,
    #[case] expected_status: TaskStatus,
) {
    let task = db.created(&ALICE, minutes(0)).await;

    let reviewed = db
        .review(task.id, &BOB, verdict, minutes(1))
        .await
        .expect("should execute review")
        .expect("task should be reviewed");

    assert_eq!(
        reviewed,
        AdminTask {
            status: expected_status,
            reviewed_at: Some(minutes(1)),
            reviewed_by_user_id: Some(BOB.id.to_string()),
            reviewed_by_display_name: Some(BOB.display_name.to_string()),
            review_comment: Some(REVIEW_COMMENT.to_string()),
            ..task
        }
    );
    db.assert_stored(&reviewed, "review must be stored").await;
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_fails_for_unknown_task(#[future(awt)] db: TestDatabase) {
    assert_eq!(
        db.review(Uuid::new_v4(), &BOB, ReviewVerdict::Approve, minutes(0))
            .await
            .expect("should execute review"),
        None
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_review_fails_when_not_created(
    #[future(awt)] db: TestDatabase,
    #[values(State::Approved, State::Withdrawn)] state: State,
) {
    let task = db.task_in(state).await;

    assert_eq!(
        db.review(task.id, &CAROL, ReviewVerdict::Reject, minutes(5))
            .await
            .expect("should execute review"),
        None
    );
    db.assert_stored(&task, "earlier state must stick").await;
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_store_execution_result_sets_execution_fields(
    #[future(awt)] db: TestDatabase,
    #[values(true, false)] execution_successful: bool,
) {
    let approved = db.task_in(State::Approved).await;

    let executed = db
        .execute(approved.id, execution_successful, minutes(5))
        .await
        .expect("should store execution result")
        .expect("task should be executed");

    assert_eq!(
        executed,
        AdminTask {
            executed_at: Some(minutes(5)),
            execution_successful: Some(execution_successful),
            execution_details: Some(EXECUTION_DETAILS.to_string()),
            ..approved
        }
    );
    db.assert_stored(&executed, "execution result must be stored").await;
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_store_execution_result_fails_for_unknown_task(#[future(awt)] db: TestDatabase) {
    assert_eq!(
        db.execute(Uuid::new_v4(), true, minutes(0))
            .await
            .expect("should store execution result"),
        None
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_store_execution_result_fails_when_not_approved_or_already_executed(
    #[future(awt)] db: TestDatabase,
    #[values(State::Rejected, State::Executed)] state: State,
) {
    let task = db.task_in(state).await;

    assert_eq!(
        db.execute(task.id, false, minutes(5))
            .await
            .expect("should store execution result"),
        None
    );
    db.assert_stored(&task, "earlier state must stick").await;
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_open_tasks_returns_own_created_tasks_oldest_first(
    #[future(awt)]
    #[from(clean_db)]
    db: TestDatabase,
) {
    let newer = db.created(&ALICE, minutes(1)).await;
    let older = db.created(&ALICE, minutes(0)).await;
    let _of_other_user = db.created(&BOB, minutes(0)).await;
    let withdrawn = db.created(&ALICE, minutes(0)).await;
    db.withdrawn(withdrawn.id, minutes(2)).await;

    assert_eq!(
        db.open_tasks(&ALICE).await.expect("should list open tasks"),
        vec![older, newer]
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_history_returns_finished_tasks_created_or_reviewed_by_user_oldest_first(
    #[future(awt)]
    #[from(clean_db)]
    db: TestDatabase,
) {
    let reviewed_by_alice = db.created(&CAROL, minutes(2)).await;
    let reviewed_by_alice = db
        .reviewed(reviewed_by_alice.id, &ALICE, ReviewVerdict::Reject, minutes(3))
        .await;

    let created_by_alice_and_approved = db.created(&ALICE, minutes(1)).await;
    let created_by_alice_and_approved = db
        .reviewed(
            created_by_alice_and_approved.id,
            &BOB,
            ReviewVerdict::Approve,
            minutes(3),
        )
        .await;

    let withdrawn_by_alice = db.created(&ALICE, minutes(0)).await;
    let withdrawn_by_alice = db.withdrawn(withdrawn_by_alice.id, minutes(3)).await;

    let _still_open = db.created(&ALICE, minutes(0)).await;
    let reviewed_by_other_user = db.created(&CAROL, minutes(0)).await;
    db.reviewed(reviewed_by_other_user.id, &BOB, ReviewVerdict::Approve, minutes(3))
        .await;

    assert_eq!(
        db.history(&ALICE).await.expect("should list task history"),
        vec![withdrawn_by_alice, created_by_alice_and_approved, reviewed_by_alice]
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_reviewable_returns_authorized_tasks_of_other_users_oldest_first(
    #[future(awt)]
    #[from(clean_db)]
    db: TestDatabase,
) {
    let newer = db.created(&CAROL, minutes(1)).await;
    let older = db.created(&BOB, minutes(0)).await;
    let _own = db.created(&ALICE, minutes(0)).await;
    let _revoke_solution = db.created_of_type(&CAROL, minutes(2), TaskType::RevokeSolution).await;
    let already_reviewed = db.created(&BOB, minutes(0)).await;
    db.reviewed(already_reviewed.id, &CAROL, ReviewVerdict::Approve, minutes(2))
        .await;
    let withdrawn = db.created(&BOB, minutes(0)).await;
    db.withdrawn(withdrawn.id, minutes(2)).await;

    let reviewable = db
        .reviewable(&ALICE, vec![TaskType::RevokeWallet])
        .await
        .expect("should list reviewable tasks");

    assert_eq!(reviewable, vec![older, newer]);
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_reviewable_with_no_authorized_task_types_returns_empty(#[future(awt)] db: TestDatabase) {
    db.created(&BOB, minutes(0)).await;

    let reviewable = db
        .reviewable(&ALICE, vec![])
        .await
        .expect("should list reviewable tasks");

    assert!(reviewable.is_empty());
}
