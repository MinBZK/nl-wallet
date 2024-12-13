# Update Policy Server

The Update Policy Server is a server providing the update policy for the NL Wallet at a specific time. It has a single
endpoint "`/update/v1/update-policy`" that returns the update policy in JSON format.

## Configuration

The update policy is configured in the server settings, and only loaded at startup. A change in the policy requires a
restart of the server. The configuration is stored in the `update_policy_server.toml` file and may contain
configurations that apply to the future. The configuration has the following format:

```toml
# server configuration
# ip = ...
# port = ...

[update_policy]
# VERSION_REQ = STATE
# or
# VERSION_REQ = { STATE = DATE_TIME }
# Example:
"<=0.1.0" = "Block"
">0.1.0, <0.2.1" = "Recommend"
"=0.2.2" = { "Notify" = "2021-12-31T23:59:59Z" }
```

A version requirement is a string that describes the (range of) version(s) that a state applies to. Allowed comparison
operators are "=", "<", "<=", ">" and ">=". The version must be valid semver. Multiple requirements can be specified,
separated by a comma. The state is a string identifying the state a version is in. It must be either "Block", "Notify"
or "Recommend". The "Ok" and "Warn" state are derived states and not used in the configuration. A version is "Ok" if it
is not any other state, it gets the state "Warn" if it will be blocked within a week and the specified state otherwise
(given that the specified datetime stamp is in the past). If a version matches multiple requirements, the most strict
one is used.
