use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

pub trait PinPolicyEvaluator {
    fn evaluate(
        &self,
        attempts: u8,
        last_failed_pin: Option<DateTime<Utc>>,
        current_datetime: DateTime<Utc>,
    ) -> PinPolicyEvaluation;
}

#[derive(Debug, PartialEq, Eq)]
pub enum PinPolicyEvaluation {
    Failed {
        attempts_left_in_round: u8,
        is_final_round: bool,
    },
    Timeout {
        timeout: Duration,
    },
    InTimeout {
        timeout: Duration,
    },
    BlockedPermanently,
}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::model::pin_policy::PinPolicyEvaluation;
    use crate::model::pin_policy::PinPolicyEvaluator;
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::Utc;

    pub struct FailingPinPolicy;
    impl PinPolicyEvaluator for FailingPinPolicy {
        fn evaluate(
            &self,
            _attempts: u8,
            _last_failed_pin: Option<DateTime<Utc>>,
            _current_datetime: DateTime<Utc>,
        ) -> PinPolicyEvaluation {
            PinPolicyEvaluation::Failed {
                attempts_left_in_round: 3,
                is_final_round: false,
            }
        }
    }

    pub struct TimeoutPinPolicy;
    impl PinPolicyEvaluator for TimeoutPinPolicy {
        fn evaluate(
            &self,
            _attempts: u8,
            _last_failed_pin: Option<DateTime<Utc>>,
            _current_datetime: DateTime<Utc>,
        ) -> PinPolicyEvaluation {
            PinPolicyEvaluation::Timeout {
                timeout: Duration::seconds(60),
            }
        }
    }
}
