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
