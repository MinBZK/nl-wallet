# Accepted false positives

This file lists findings that have been reviewed and accepted as **not
actionable** — either genuine false positives or documented, accepted-risk
design decisions. It is passed to `/triage` via `--fp-rules`, which appends
the rules below to the verifier's exclusion list: a finding that matches one
of them is marked `false_positive` and does not resurface as a new issue.

## How to add an entry

Add a new `### N. <short title>` section. Write the rule as prose the
verifier can pattern-match a finding against: name the **file/component** and
the **class of finding** it covers, scope it as narrowly as the accepted risk
actually is (a whole file, a specific connection, a specific function — not a
whole category), and state **why** it is accepted, with a ticket reference
where one exists. Do not match on finding ids (`f019`, …) — those are
per-run and not stable across scans; match on code location and the nature
of the finding instead.

Keep each rule self-contained: the triage verifier sees only the text below,
not this repository's history, so everything it needs to make the call must
be in the rule.

---

## Rules

### 1. pid_issuer BRP client — plain HTTP to brpproxy (missing-encryption)

The pid_issuer's BRP client
(`wallet_core/wallet_server/pid_issuer/src/pid/brp/client.rs`) intentionally
permits a plain `http://` base URL for the connection to the brpproxy, and
production deploys it as `http://brpproxy/`. This is a documented, accepted
design decision: the service is meant to be run on an internal network, as
noted in the code ("Note that this specifically allows HTTP, as this service
is meant to be run on an internal network."), and the residual risk is
accepted and tracked as PVW-5612.

Findings that flag this connection for missing TLS/encryption are
FALSE_POSITIVE, including: cleartext transmission of the BSN or other data to
the BRP proxy, the absence of `.https_only(true)` on the BRP reqwest client,
or the BRP/brpproxy base URL being allowed to use `http://`. This rule is
scoped to the pid_issuer → brpproxy BRP connection only; it does not apply to
any other outbound connection, nor to any wallet-facing or internet-facing
endpoint.
