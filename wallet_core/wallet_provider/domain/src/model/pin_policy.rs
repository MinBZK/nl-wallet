use chrono::{DateTime, Duration, Local};

pub trait PinPolicyEvaluator {
    fn evaluate(
        &self,
        attempts: u8,
        last_failed_pin: Option<DateTime<Local>>,
        current_datetime: DateTime<Local>,
    ) -> PinPolicyEvaluation;
}

#[derive(Debug, PartialEq, Eq)]
pub enum PinPolicyEvaluation {
    Failed { attempts_left: u8, is_final_attempt: bool },
    Timeout { timeout: Duration },
    InTimeout { timeout: Duration },
    BlockedPermanently,
}

#[cfg(feature = "mock")]
pub mod mock {
    use crate::model::pin_policy::{PinPolicyEvaluation, PinPolicyEvaluator};
    use chrono::{DateTime, Duration, Local};

    pub struct FailingPinPolicy;
    impl PinPolicyEvaluator for FailingPinPolicy {
        fn evaluate(
            &self,
            _attempts: u8,
            _last_failed_pin: Option<DateTime<Local>>,
            _current_datetime: DateTime<Local>,
        ) -> PinPolicyEvaluation {
            PinPolicyEvaluation::Failed {
                attempts_left: 3,
                is_final_attempt: false,
            }
        }
    }

    pub struct TimeoutPinPolicy;
    impl PinPolicyEvaluator for TimeoutPinPolicy {
        fn evaluate(
            &self,
            _attempts: u8,
            _last_failed_pin: Option<DateTime<Local>>,
            _current_datetime: DateTime<Local>,
        ) -> PinPolicyEvaluation {
            PinPolicyEvaluation::Timeout {
                timeout: Duration::seconds(60),
            }
        }
    }
}
