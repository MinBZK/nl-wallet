use chrono::{DateTime, Duration, Local};

use wallet_provider_domain::model::pin_policy::{PinPolicyEvaluation, PinPolicyEvaluator};

pub struct PinPolicy {
    rounds: u8,
    attempts_per_round: u8,
    timeouts: Vec<Duration>,
}

impl PinPolicy {
    pub fn new(rounds: u8, attempts_per_round: u8, timeouts: Vec<Duration>) -> Self {
        assert!(
            timeouts.len() == usize::from(rounds) - 1,
            "a timeout should be provided for all rounds but the first"
        );

        Self {
            rounds,
            attempts_per_round,
            timeouts,
        }
    }

    fn current_round(&self, attempts: u8) -> u8 {
        assert!(attempts > 0);

        match (attempts / self.attempts_per_round, attempts % self.attempts_per_round) {
            (0, _) => 1,
            (x, 0) => x,
            (x, _) => x + 1,
        }
    }

    fn is_final_attempt(&self, attempts: u8) -> bool {
        assert!(attempts > 0);

        self.attempts_left(attempts) == 1 && self.current_round(attempts) == self.rounds
    }

    fn is_blocked(&self, attempts: u8) -> bool {
        assert!(attempts > 0);

        let total_attempts = self.rounds * self.attempts_per_round;
        attempts >= total_attempts
    }

    fn attempts_left(&self, attempts: u8) -> u8 {
        assert!(attempts > 0);

        if self.rounds == 1 {
            return self.attempts_per_round - attempts;
        }

        match attempts % self.rounds {
            0 => {
                if self.is_blocked(attempts) {
                    0
                } else {
                    self.attempts_per_round
                }
            }
            x => self.attempts_per_round - x,
        }
    }

    fn current_timeout(&self, attempts: u8) -> Option<Duration> {
        assert!(attempts > 0);

        let i = usize::from(attempts / self.attempts_per_round);

        if !self.is_blocked(attempts) && attempts > 1 && i > 0 {
            self.timeouts.get(i - 1).copied()
        } else {
            None
        }
    }
}

impl PinPolicyEvaluator for PinPolicy {
    fn evaluate(
        &self,
        attempts: u8,
        last_failed_pin: Option<DateTime<Local>>,
        current_datetime: DateTime<Local>,
    ) -> PinPolicyEvaluation {
        let is_first_attempt = last_failed_pin.is_none() && attempts == 1;
        let has_failed_earlier = last_failed_pin.is_some() && attempts > 1;
        assert!(
            is_first_attempt || has_failed_earlier,
            "cannot evaluate pin policy for inconsistent starting point"
        );

        if self.is_blocked(attempts) {
            return PinPolicyEvaluation::BlockedPermanently;
        }

        if let (Some(last_failed), Some(timeout)) = (last_failed_pin, self.current_timeout(attempts)) {
            let already_in_timeout = last_failed + timeout > current_datetime;
            let end_of_round = self.attempts_per_round == self.attempts_left(attempts);
            let start_of_next_round = self.attempts_per_round == self.attempts_left(attempts) + 1;

            if end_of_round {
                return PinPolicyEvaluation::Timeout { timeout };
            }

            if already_in_timeout && start_of_next_round {
                return PinPolicyEvaluation::InTimeout { timeout };
            }
        }

        PinPolicyEvaluation::Failed {
            attempts_left: self.attempts_left(attempts),
            is_final_attempt: self.is_final_attempt(attempts),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use chrono::{Duration, Local};
    use rstest::rstest;

    use wallet_provider_domain::model::pin_policy::{PinPolicyEvaluation, PinPolicyEvaluator};

    use crate::pin_policy::PinPolicy;

    #[test]
    #[should_panic]
    fn evaluate_should_panic_attempt_is_zero() {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::seconds).collect());
        policy.evaluate(0, None, Local::now());
    }

    #[test]
    #[should_panic]
    fn evaluate_should_panic_when_first_attempt_and_last_failed_time() {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::seconds).collect());
        policy.evaluate(1, Some(Local::now()), Local::now());
    }

    #[test]
    #[should_panic]
    fn evaluate_should_panic_when_second_attempt_but_no_last_failed_time() {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::seconds).collect());
        policy.evaluate(2, None, Local::now());
    }

    #[test]
    fn test_evaluate() {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::hours).collect());

        assert_eq!(
            PinPolicyEvaluation::BlockedPermanently,
            policy.evaluate(16, Some(Local::now()), Local::now()),
        );

        assert_eq!(
            PinPolicyEvaluation::BlockedPermanently,
            policy.evaluate(100, Some(Local::now()), Local::now())
        );

        assert_eq!(
            PinPolicyEvaluation::Failed {
                attempts_left: 3,
                is_final_attempt: false
            },
            policy.evaluate(1, None, Local::now())
        );

        assert_eq!(
            PinPolicyEvaluation::Failed {
                attempts_left: 2,
                is_final_attempt: false
            },
            policy.evaluate(2, Some(Local::now() - Duration::hours(1)), Local::now())
        );

        assert_eq!(
            PinPolicyEvaluation::Failed {
                attempts_left: 1,
                is_final_attempt: false
            },
            policy.evaluate(3, Some(Local::now() - Duration::hours(1)), Local::now())
        );

        assert_eq!(
            PinPolicyEvaluation::Timeout {
                timeout: Duration::hours(1),
            },
            policy.evaluate(4, Some(Local::now() - Duration::hours(1)), Local::now())
        );

        assert_matches!(
            policy.evaluate(5, Some(Local::now() - Duration::minutes(30)), Local::now()),
            PinPolicyEvaluation::InTimeout { timeout: t } if t == Duration::hours(1)
        );

        assert_eq!(
            PinPolicyEvaluation::Timeout {
                timeout: Duration::hours(3),
            },
            policy.evaluate(12, Some(Local::now() - Duration::hours(3)), Local::now())
        );

        assert_eq!(
            PinPolicyEvaluation::Failed {
                attempts_left: 1,
                is_final_attempt: true
            },
            policy.evaluate(15, Some(Local::now() - Duration::hours(3)), Local::now())
        );

        assert_matches!(
            policy.evaluate(4, Some(Local::now() - Duration::minutes(30)), Local::now()),
            PinPolicyEvaluation::Timeout {
                timeout: t,
            } if t == Duration::hours(1)
        );

        assert_matches!(
            policy.evaluate(8, Some(Local::now() - Duration::minutes(30)), Local::now()),
            PinPolicyEvaluation::Timeout {
                timeout: t,
            } if t == Duration::hours(2)
        );

        assert_matches!(
            policy.evaluate(12, Some(Local::now() - Duration::hours(1)), Local::now()),
            PinPolicyEvaluation::Timeout {
                timeout: t,
            } if t == Duration::hours(3)
        );

        assert_matches!(
            policy.evaluate(13, Some(Local::now() - Duration::hours(1)), Local::now()),
            PinPolicyEvaluation::InTimeout {
                timeout: t,
            } if t == Duration::hours(3)
        );
    }

    #[rstest]
    #[case(1, 1)]
    #[case(1, 2)]
    #[case(1, 3)]
    #[case(1, 4)]
    #[case(2, 5)]
    #[case(2, 6)]
    #[case(2, 7)]
    #[case(2, 8)]
    #[case(3, 9)]
    #[case(3, 10)]
    #[case(3, 11)]
    #[case(3, 12)]
    #[case(4, 13)]
    #[case(4, 14)]
    #[case(4, 15)]
    #[case(4, 16)]
    fn should_return_current_round_for_attempts_for_4_rounds_and_4_attempts(
        #[case] expected_round: u8,
        #[case] attempts: u8,
    ) {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::seconds).collect());
        assert_eq!(expected_round, policy.current_round(attempts));
    }

    #[rstest]
    #[case(1, 1)]
    fn should_return_current_round_for_attempts_for_1_round_and_1_attempts(
        #[case] expected_round: u8,
        #[case] attempts: u8,
    ) {
        let policy = PinPolicy::new(1, 1, vec![]);
        assert_eq!(expected_round, policy.current_round(attempts));
    }

    #[rstest]
    #[case(1, 1)]
    #[case(1, 2)]
    fn should_return_current_round_for_attempts_for_1_round_and_2_attempts(
        #[case] expected_round: u8,
        #[case] attempts: u8,
    ) {
        let policy = PinPolicy::new(1, 2, vec![]);
        assert_eq!(expected_round, policy.current_round(attempts));
    }

    #[rstest]
    #[case(1, 1)]
    #[case(2, 2)]
    fn should_return_current_round_for_attempts_for_2_round_and_1_attempts(
        #[case] expected_round: u8,
        #[case] attempts: u8,
    ) {
        let policy = PinPolicy::new(2, 1, (1..2).map(Duration::seconds).collect());
        assert_eq!(expected_round, policy.current_round(attempts));
    }

    #[rstest]
    #[case(None, 1)]
    fn should_return_current_timeout_for_1_round_1_attempts(
        #[case] expected_timeout_in_sec: Option<i64>,
        #[case] attempts: u8,
    ) {
        let policy = PinPolicy::new(1, 1, vec![]);
        assert_eq!(
            expected_timeout_in_sec,
            policy.current_timeout(attempts).map(|d| d.num_seconds())
        );
    }

    #[rstest]
    #[case(None, 1)]
    #[case(None, 2)]
    fn should_return_current_timeout_for_1_round_2_attempts(
        #[case] expected_timeout_in_sec: Option<i64>,
        #[case] attempts: u8,
    ) {
        let policy = PinPolicy::new(1, 2, vec![]);
        assert_eq!(
            expected_timeout_in_sec,
            policy.current_timeout(attempts).map(|d| d.num_seconds())
        );
    }

    #[rstest]
    #[case(None, 1)]
    #[case(None, 2)]
    #[case(None, 3)]
    #[case(Some(1), 4)]
    #[case(Some(1), 5)]
    #[case(Some(1), 6)]
    #[case(Some(1), 7)]
    #[case(Some(2), 8)]
    #[case(Some(2), 9)]
    #[case(Some(2), 11)]
    #[case(Some(3), 12)]
    #[case(Some(3), 13)]
    #[case(Some(3), 15)]
    #[case(None, 16)]
    fn should_return_current_timeout(#[case] expected_timeout_in_sec: Option<i64>, #[case] attempts: u8) {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::seconds).collect());
        assert_eq!(
            expected_timeout_in_sec,
            policy.current_timeout(attempts).map(|d| d.num_seconds())
        );
    }

    #[rstest]
    #[case(3, 1)]
    #[case(2, 2)]
    #[case(1, 3)]
    #[case(4, 4)]
    #[case(3, 5)]
    #[case(2, 6)]
    #[case(1, 7)]
    #[case(4, 8)]
    #[case(3, 9)]
    #[case(1, 15)]
    #[case(0, 16)]
    fn should_indicate_remaining_attempts(#[case] expected_remaining: u8, #[case] attempts: u8) {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::seconds).collect());
        assert_eq!(expected_remaining, policy.attempts_left(attempts));
    }

    #[rstest]
    #[case(false, 1)]
    fn should_indicate_if_final_attempt_for_1_round_1_attempt(#[case] expected_is_final: bool, #[case] attempts: u8) {
        let policy = PinPolicy::new(1, 1, vec![]);
        assert_eq!(expected_is_final, policy.is_final_attempt(attempts));
    }

    #[rstest]
    #[case(true, 1)]
    #[case(false, 2)]
    fn should_indicate_if_final_attempt_for_1_round_2_attempt(#[case] expected_is_final: bool, #[case] attempts: u8) {
        let policy = PinPolicy::new(1, 2, vec![]);
        assert_eq!(expected_is_final, policy.is_final_attempt(attempts));
    }

    #[rstest]
    #[case(false, 1)]
    #[case(false, 2)]
    #[case(false, 3)]
    #[case(false, 4)]
    #[case(false, 5)]
    #[case(false, 7)]
    #[case(false, 8)]
    #[case(true, 15)]
    #[case(false, 16)]
    fn should_indicate_if_final_attempt(#[case] expected_is_final: bool, #[case] attempts: u8) {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::seconds).collect());
        assert_eq!(expected_is_final, policy.is_final_attempt(attempts));
    }

    #[rstest]
    #[case(true, 1)]
    fn should_indicate_if_blocked_for_1_round_1_attempt(#[case] expected_is_blocked: bool, #[case] attempts: u8) {
        let policy = PinPolicy::new(1, 1, vec![]);
        assert_eq!(expected_is_blocked, policy.is_blocked(attempts));
    }

    #[rstest]
    #[case(false, 1)]
    #[case(true, 2)]
    fn should_indicate_if_blocked_for_1_round_2_attempt(#[case] expected_is_blocked: bool, #[case] attempts: u8) {
        let policy = PinPolicy::new(1, 2, vec![]);
        assert_eq!(expected_is_blocked, policy.is_blocked(attempts));
    }

    #[rstest]
    #[case(false, 1)]
    #[case(false, 3)]
    #[case(false, 4)]
    #[case(false, 7)]
    #[case(false, 8)]
    #[case(false, 15)]
    #[case(true, 16)]
    fn should_indicate_if_blocked(#[case] expected_is_blocked: bool, #[case] attempts: u8) {
        let policy = PinPolicy::new(4, 4, (1..4).map(Duration::seconds).collect());
        assert_eq!(expected_is_blocked, policy.is_blocked(attempts));
    }
}
