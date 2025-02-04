This Definition of Done (the DoD) is a mutual agreement between the development
team and the product owner. It is a effectively a list of things we consider
necessary to do when working on issues and changes.

[[_TOC_]]

## Things we check

We check the following things before considering an issue or change "done". This
list is also included in our default MR template, so it gets checked every time
we merge something to main.

### Has a related issue ticket with a correct issue type and description

Blah.

### Is implemented as described in the aforementioned issue ticket

Blah.

### Implements any tests as specified in the related issue ticket

Blah.

### Has unit- and integration tests which prove its correctness

Blah.

### Implements any screens according to Figma design (if any)

Blah.

### Is featured and described in the upcoming release notes

Blah.

### Has any breaking changes documented in the release notes

Blah.

### Has any upgrade steps documented in the release notes

Blah.

### Has updated any README.md files related to these changes

Blah.

### Has any API changes reflected in the OpenAPI YAML documention

Blah.

### Has updated any related documentation in the documentation folder

Blah.

### Has any incurred technical debt reflected by a TODO comment in code

Blah.

### Does not contain commits with personally identifiable information

Blah.

### Does not contain commits with secrets or internal identifiers

Blah.

### Does not contain commits with copyrighted files or data

Blah.

### Does not contain commits with inappropriate contents

Blah.

### Is in compliance with our coding and quality standards

Blah.

### Succesfully passes the relevant Sonar quality gate

Blah.

### Configured any relevant CI/CD pipeline variables

Blah.

### Configured any relevant CI/CD pipeline changes

Blah.

### Configured any relevant Kubernetes objects

Blah.











---------------------- old stuff below ----------------

RATIONALE AND/OR IMPORTANT IMPLEMENTATION DECISIONS FOR THE REVIEWER

## Review checklist

- [ ] This MR is implemented according to the specifications as described in: ([PVW-XXXX](https://JIRA_LINK) | [CAP-XXXX](https://CAP_LINK))
    - This MR implements all tests as specified
    - The specifications are updated according to new insights discovered during development
    - This MR implements screens according to the Figma design
- [ ] All relevant documentation is updated
    - General documentation in the [documentation](./documentation/) folder
    - Relying party documentation in [documentation/relying-party.md](./documentation/relying-party.md)
    - OpenAPI documentation in [documentation/api](./documentation/api/)
    - any README.md
- [ ] Deployment files and CI/CD pipelines are updated
    - including Gitlab variables
    - including deployment configuration and secrets
- [ ] [Release Notes](./documentation/release-notes/) for the upcoming version contains an entry for this MR
    - Breaking Changes
    - Upgrade Steps
- [ ] This MR does not contain commits with code or commit messages that contain any of the following
    - Personal information
    - Copyrighted files or data
    - Secrets or internal identifiers such as hostnames, project IDs, passwords, private keys, etc.
    - Sensitive internal details that are not yet ready for publication
    - Other unexpected, strange or inappropriate things, that we don't want to make public
- [ ] This MR is implemented according to our Definition Of Done
    - The source code is in compliance with the coding standards
    - The source code does not contain ‘blocker’ and ‘critical’ violations (Sonar)
    - The source code does not contain ‘major’ violations (Sonar) unless accepted by quality manager and is planned for solving
    - Security findings have been resolved, or at least provided with impact and/or resolution time
    - Incurred technical debt has been documented
