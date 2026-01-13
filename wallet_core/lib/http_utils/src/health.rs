//! Following MicroProfile Health [specification](https://github.com/microprofile/microprofile-health/blob/main/spec/src/main/asciidoc/protocol-wireformat.asciidoc):

use async_trait::async_trait;
use serde::Serialize;

#[cfg(feature = "server")]
pub use router::create_health_router;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HealthStatus {
    DOWN,
    UP,
}

#[async_trait]
pub trait HealthChecker: Send + Sync {
    fn name(&self) -> &'static str;
    async fn status(&self) -> Result<HealthStatus, Box<dyn std::error::Error + Send + Sync>>;
}

#[cfg(feature = "server")]
mod router {
    use std::collections::HashMap;
    use std::sync::Arc;

    use axum::Json;
    use axum::Router;
    use axum::extract::State;
    use axum::response::IntoResponse;
    use axum::response::Response;
    use axum::routing::get;
    use futures::future::join_all;
    use http::StatusCode;
    use itertools::Itertools;
    use serde::Serialize;

    use super::HealthChecker;
    use super::HealthStatus;

    #[derive(Debug, Serialize)]
    struct HealthCheck {
        name: &'static str,
        status: HealthStatus,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        data: HashMap<&'static str, String>, // value can be also bool or number,
    }

    #[derive(Debug, Serialize)]
    struct HealthResponse {
        status: HealthStatus,
        checks: Vec<HealthCheck>,
    }

    #[derive(Clone)]
    struct HealthState {
        checkers: Arc<Vec<Box<dyn HealthChecker>>>,
    }

    /// Create MicroProfile health router.
    ///
    /// # Panics
    /// The function panics if multiple checkers are passed with the same name.
    pub fn create_health_router(checkers: impl IntoIterator<Item = Box<dyn HealthChecker>>) -> Router {
        let checkers = checkers.into_iter().collect_vec();

        let duplicates = checkers
            .iter()
            .counts_by(|c| c.name())
            .iter()
            .filter_map(|(name, count)| (*count > 1).then_some(*name))
            .collect_vec();
        if !duplicates.is_empty() {
            panic!("Multiple checkers found with the same name: {:?}", duplicates)
        }

        let state = HealthState {
            checkers: Arc::new(checkers),
        };
        Router::new()
            .route("/health", get(health_check))
            .route("/health/ready", get(health_check))
            .route("/health/started", get(health_check))
            .route("/health/live", get(empty_check))
            .with_state(state)
    }

    async fn empty_check() -> Response {
        Json(HealthResponse {
            status: HealthStatus::UP,
            checks: Vec::new(),
        })
        .into_response()
    }

    async fn health_check(State(state): State<HealthState>) -> Response {
        let results = join_all(state.checkers.iter().map(|checker| async {
            let name = checker.name();
            let result = checker
                .status()
                .await
                .inspect_err(|err| tracing::error!("Error with health check `{}`: {}", name, err));
            (name, result)
        }))
        .await;

        // Create checks
        let checks = results
            .into_iter()
            .map(|result| match result {
                (name, Ok(status)) => HealthCheck {
                    name,
                    status,
                    data: HashMap::new(),
                },
                (name, Err(err)) => HealthCheck {
                    name,
                    status: HealthStatus::DOWN,
                    data: [("error", err.to_string())].into(),
                },
            })
            .collect_vec();

        // Determine status from checks
        let (status, status_code) = if checks.iter().all(|check| check.status == HealthStatus::UP) {
            (HealthStatus::UP, StatusCode::OK)
        } else {
            (HealthStatus::DOWN, StatusCode::SERVICE_UNAVAILABLE)
        };

        (
            status_code,
            [(http::header::CONTENT_TYPE, "application/json")],
            Json(HealthResponse { status, checks }),
        )
            .into_response()
    }

    #[cfg(test)]
    mod tests {
        use std::panic::catch_unwind;

        use async_trait::async_trait;
        use axum::body;
        use rstest::rstest;
        use serde_json::Value;
        use serde_json::json;

        use super::*;

        #[derive(Debug, Clone, thiserror::Error)]
        #[error("{0}")]
        struct TestError(String);

        struct TestChecker {
            name: &'static str,
            result: Result<HealthStatus, TestError>,
        }

        impl TestChecker {
            fn ok(name: &'static str, status: HealthStatus) -> Self {
                Self {
                    name,
                    result: Ok(status),
                }
            }

            fn err(name: &'static str, reason: &'static str) -> Self {
                Self {
                    name,
                    result: Err(TestError(reason.to_string())),
                }
            }
        }

        #[async_trait]
        impl HealthChecker for TestChecker {
            fn name(&self) -> &'static str {
                self.name
            }

            async fn status(&self) -> Result<HealthStatus, Box<dyn std::error::Error + Send + Sync>> {
                match &self.result {
                    Ok(status) => Ok(*status),
                    Err(err) => Err(Box::new(err.clone())),
                }
            }
        }

        #[test]
        fn test_create_health_router_checks_duplicates() {
            let result = catch_unwind(|| {
                let a1 = Box::new(TestChecker::ok("a", HealthStatus::UP)) as Box<dyn HealthChecker>;
                let a2 = Box::new(TestChecker::ok("a", HealthStatus::UP)) as Box<dyn HealthChecker>;
                _ = create_health_router([a1, a2])
            });
            assert_eq!(
                result
                    .expect_err("health router should panic")
                    .downcast_ref::<String>()
                    .unwrap(),
                "Multiple checkers found with the same name: [\"a\"]"
            );
        }

        #[tokio::test]
        #[rstest]
        #[case::empty(vec![], StatusCode::OK, json!({"status": "UP", "checks": []}))]
        #[case::one_up(
            vec![TestChecker::ok("a", HealthStatus::UP)],
            StatusCode::OK,
            json!({"status": "UP", "checks": [{"name": "a", "status": "UP"}]})
        )]
        #[case::one_down(
            vec![TestChecker::ok("b", HealthStatus::DOWN)],
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"status": "DOWN", "checks": [{"name": "b", "status": "DOWN"}]})
        )]
        #[case::one_up_one_down(
            vec![TestChecker::ok("a", HealthStatus::UP),
            TestChecker::ok("b", HealthStatus::DOWN)],
            StatusCode::SERVICE_UNAVAILABLE,
            json!({"status": "DOWN", "checks": [{"name": "a", "status": "UP"}, {"name": "b", "status": "DOWN"}]})
        )]
        #[case::one_up_one_err(
            vec![TestChecker::ok("a", HealthStatus::UP), TestChecker::err("c", "reason")],
            StatusCode::SERVICE_UNAVAILABLE,
            json!({
                "status": "DOWN",
                "checks": [{"name": "a", "status": "UP"}, {"name": "c", "status": "DOWN", "data": {"error": "reason"}}]
            })
        )]
        async fn test_health_check(
            #[case] checkers: Vec<TestChecker>,
            #[case] expected_status_code: StatusCode,
            #[case] expected_body: Value,
        ) {
            let checkers = checkers.into_iter().map(|c| Box::new(c) as Box<_>).collect_vec();
            let state = HealthState {
                checkers: Arc::new(checkers),
            };

            let response = health_check(State(state)).await;
            assert_eq!(response.status(), expected_status_code);
            assert_eq!(response.headers()["content-type"], "application/json");

            let body = body::to_bytes(response.into_body(), 1024).await.unwrap();
            let body = serde_json::from_slice::<Value>(&body).unwrap();
            assert_eq!(body, expected_body);
        }
    }
}
