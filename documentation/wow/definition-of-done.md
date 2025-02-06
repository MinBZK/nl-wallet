TODO: Rewrite everything, but not before there is consensus about the MR template

This Definition of Done (the DoD) is a mutual agreement between the development
team and the product owner. team and the product owner. It is essentially a list
of conditions that a change to the code should adhere to before being considered
ready to be merged into the `main` branch.

[[_TOC_]]

## In general

* Written knowledge is saved and kept up to date
* We create use-cases for our functional design purposes
* We keep our architecture and project documents, like the SAD and PSA
  up-to-date and in-sync with the current implementation
* A task is done when all acceptance criteria have been met (see things
  we check below)
* A done task can be demonstrated during Sprint Review
* When we have a security finding, we aim to resolve it quickly or
  at least provide our stakeholders with an impact statement and/or
  resolution time
* Tasks are done by way of a pull- or merge-request and are reviewed
  by at least one colleague who specifically checks the list of items
  described below

## Things we check every pull- or merge request

We check the following things before considering an issue or change "done". This
list is also included in our default pull- or merge-template, so it gets checked
every time we merge something to main.

### Has a related issue ticket with a correct issue type and description

A change should have a related issue ticket in our issue tracker. The issue
should have a correct issue type (bug, feature, story, etc) and a clear
description that states what the work entails.

Ideally, it has any relevant epic associated and possibly relevant tags which
make it easier to query this issue later on.

### Is implemented as described in the aforementioned issue ticket

The aforementioned issue contains a clear description of what needs to be done.

It should be made sure that whatever is the goal of the issue, is met in the
implementation (i.e., the code as implemented realizes what the issue states).

### Implements any tests as specified in the related issue ticket

The issue ticket might specify explicitly certain things that we want to test.

Make sure that these explicitly specified tests are actually added and pass
succesfully.

### Has unit- and integration tests which prove its correctness

Any additional code needs to be covered by tests which prove the correctness of
the implementation. In general, we strive for reasonable coverage. In general,
we prefer not to lower coverage with any given merge to main.

### Implements any screens according to Figma design (if any)

When there is work to be done on our GUI, we usually specify this work upfront
by means of Figma. Any changes to our GUI, or newly implemented screens should
be in accordance with those designs in Figma.

### Is featured and described in the upcoming release notes

Fixed bugs, implemented features and done stories are listed in our release
notes. Only really small tasks may be considered for omittance.

### Has any breaking changes documented in the release notes

If a change introduces breakage, we need to document it in the release notes.

Specifically, we need to document why the breakage occurs and what a person can
do to mitigate the breakage.

### Has any upgrade steps documented in the release notes

If a change requires specific upgrade steps, they need to be specified in the
release notes.

### Has updated any README.md files related to these changes

We have many components in our code base. These might have a README.md document
that describes their function. When you work on something that has a README.md,
make sure that it still is correct and reflects the current workings of the
component.

### Has any API changes reflected in the OpenAPI YAML documention

We have various public and private API interfaces (specifically, we're talking
about HTTP REST based APIs in this case).

When we touch code that affects those interfaces, we need to make sure that the
documentation for these APIs is correct.

### Has updated any related documentation in the documentation folder

We have a documentation folder with all kinds of documentation about our
software. When we work on various parts of our code base, we need to make sure
that any documentation there is still up-to-date and accurate, plus reflects
any changes done to the code/functionality/etc.

### Has any incurred technical debt reflected by a TODO comment in code

From time to time we necessarily introduce (hopefully temporary) technical debt.

When such is the case, make sure that we have a TODO comment in the code that
clearly states what we need to revisit later on.

### Does not contain commits with personally identifiable information

Any commits contained in the branch should **not** contain PII.

### Does not contain commits with secrets or internal identifiers

Any commits contained in the branch should **not** contain any kind of secrets,
private keys, or internal identifiers.

### Does not contain commits with copyrighted files or data

Any commits contained in the branch should **not** contain copyrighted files
or data. Be aware of the license of any item added to the repository which was
not directly made by ourselves. If something is allowed to be distributed but
does not have a purely FOSS license, make sure it's in a `non-free` sub-folder.

### Does not contain commits with inappropriate contents

Any commits contained in the branch should **not** contain inappropriate things
like erotic material, curse-words, virii, malware, etc.

### Is in compliance with our coding and quality standards

Any code submitted is in accordance to our quality guidelines. Take into account
our `.editorconfig` and make sure you're aware of our `CONTRIBUTING.md`.

### Succesfully passes the relevant Sonar quality gate

Our CI/CD pipelines submit code to SonarQube, which is configured using quality
profiles and quality gates. SonarQube should not be screaming at you w/regards
to the code you commited.

Note that SonarQube updates can and will change quality profiles which will
introduce changes (i.e., something that was green yesterday might be red the
next day). In such a case, we usually try to fix things within the branch you
are working on, but we don't try to fix all newly detected issues *outside* of
the issue you're implementing (that would strongly suggest a new issue).

### Configures any relevant CI/CD pipeline variables

If the change touches upon any CI/CD pipeline variables, make sure they're set,
either on the GitLab group or project level, or in the GitLab YAML files.

### Configures any relevant CI/CD pipeline changes

If the change involves any CI/CD pipeline changes, make sure they're implemented
correctly and in such a way that the pipeline still executes as expected.

### Configures any relevant Kubernetes objects

Sometimes we have changes that also require changes in our target environments
which can be local or Kubernetes namespaces.

When that is the case, make sure that those required changes are done in those
target environments. For example: needed ConfigMaps or Secrets, or other types
or Kubernetes objects.

## Quality Assurance

Below are a couple of items that are related to our quality assurance efforts.
In general we try to adhere to these principals globally. Note that these items
are ortogonal to the project (a cross-cutting concern) and are not evaluated
at every pull- or merge-request. they are done continuously in parallel to the
development effort. In general:

1. The implementation adheres to the design.

2. All test cases should be updated and successfully executed (either automated
   through CI or manually). Any related functional tests should be successfully
   executed.

3. The application user interface adheres to required accesibility guidelines.

4. The code is in compliance (i.e., adhering to constraints or coverage).

5. The application is security-checked (i.e., audited, pen-tested, etc).

6. Required performance is validated.
