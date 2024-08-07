# Integration tests

This crate contains several types of integration tests that require at least a running database.
All of these tests use the `wallet` crate (e.g. the Rust core of the wallet without the `flutter_api` layer) as a client to test happy path flows against the backend services.

There are several categories of tests, which are gated behind features:
* A set of regular integration tests that require PostgreSQL and SoftHSM to be configured and running. These test the bulk of the wallet functionality and require the `integration_test` feature to be enabled. 
* An integration test for issuance that includes the DigiD connector. This requires the `digid_test` feature to be enabled.
* A performance test that is set up as a separate binary. It requires the `performance_test` feature. 

## Regular integration tests

To run these tests, perform the following steps:
* Set up the local environment using `./scripts/setup-devenv.sh`.
* Start PostgreSQL using `./scripts/start-devenv.sh postgres`.
* Run the tests from `wallet_core` using `cargo test -p tests_integration --features integration_test`.

## DigiD connector integration test

To run this test, perform the following steps:
* Set up the local environment using `./scripts/setup-devenv.sh`.
* Start PostgreSQL, the DigiD connector and the BRP proxy using `./scripts/start-devenv.sh postgres digid brpproxy`.
* Run the tests from `wallet_core` using `cargo test -p tests_integration --features digid_test`.

## Performance test

### Configuration

This test is meant to be run against an already running (external) environment. It uses the wallet default
configuration determined by `wallet/.env` (or by using environment variables).

In addition, it requires an `test_integration/.env` file containing two keys:

- `RELYING_PART_URL`: The external URL from where the disclosure flow is started. Is used as return_url.
- `WALLET_SERVER_REQUESTER_URL`: The internal URL where the disclosure session is started. Normally used by Normally
  used by the relying party server.

This file is automatically generated by the `scripts/setup-devenv.sh` script for running the performance test locally.

### Running locally

To run the test locally, the following servers need to be started:

    ./scripts/start-devenv.sh postgres wallet_provider configuration_server verification_server digid_connector brp

or shorter:

    ./scripts/start-devenv.sh postgres wp cs vs digid brp

The performance test should be built using the `allow_http_return_url` feature.

### Running externally

Change the values in `test_integration/.env` appropriately. The `allow_http_return_url` feature is not necessary (but also
doesn't hurt).
