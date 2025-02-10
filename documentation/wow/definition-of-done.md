This Definition of Done (the DoD) is a mutual agreement between the development
team and the product owner. It is essentially a list of conditions that a change
to the code should adhere to before being considered ready to be merged into the
`main` branch.

[[_TOC_]]

## In general

* Tasks are done by way of a pull- or merge-request and are reviewed
  by at least one colleague. The things we check for are described in
  this document
* A task is done when all acceptance criteria have been met and the
  work is verified by way of tests
* The implementation of the task adheres to the design
* User interface changes adhere to required accessibility guidelines
* A done task can be demonstrated during Sprint Review
* We keep our documentation up-to-date and in-sync with the current
  implementation
* We create use-cases for our functional design purposes
* When we have a security finding, we aim to resolve it quickly. We
  provide our stakeholders with an impact statement and/or resolution
  time. In general we expect to be audited, pen-tested, etc
* Required performance is validated

## Things we check every pull- or merge request

We check the following things before considering an issue or change "done". This
list is also included in our default pull- or merge-template, so it gets checked
every time we merge something to main.

### Is in compliance with our coding and quality standards

Any code submitted is in accordance to our quality guidelines. Take into account
our `.editorconfig` and make sure you're aware of our `CONTRIBUTING.md`.

### Implements and tests all acceptance criteria and/or capabilities

A change should have a related issue ticket in our issue tracker describing the
wanted behaviour or bug to-be-fixed. The issue should have a correct issue type
(bug, feature, story, etc) and a clear description that states what the work is.
Additionally, it should have any relevant epic associated and possibly relevant
tags set.

It should be made sure that whatever is the goal of the issue, is met in the
implementation (i.e., the code as implemented realizes what the issue states).

The issue ticket might specify explicitly certain things that we want to test.
Make sure that these explicitly specified tests are actually added and pass
succesfully.

Any additional code needs to be covered by tests which prove the correctness of
the implementation. In general, we strive for reasonable coverage. In general,
we prefer not to lower coverage with any given merge to main.

### Implements any screens according to our Figma designs (if applicable)

When there is work to be done on our GUI, we usually specify this work upfront
by means of Figma. Any changes to our GUI, or newly implemented screens should
be in accordance with those designs in Figma.

### Is described in release notes, with any breaking changes or upgrade steps

Fixed bugs, implemented features and done stories are listed in our release
notes. Only really small tasks may be considered for omittance.

If a change introduces breakage, it needs to documented in the release notes.
Specifically, we need to document why the breakage occurs and what a person can
do to mitigate the breakage.

If a change requires specific upgrade steps, they need to be specified in the
release notes.

### Is documented (in README.md files, OpenAPI specs, documentation folder)

We have many components in our code base. These might have a README.md document
that describes their function. When you work on something that has a README.md,
make sure that it still is correct and reflects the current workings of the
component.

We have various public and private API interfaces (specifically, we're talking
about HTTP REST based APIs in this case). When we touch code that affects those
interfaces, we need to make sure that the documentation for these APIs is
correct, meaning: the OpenAPI YAML files reflect the current state of the code.

We also have a documentation folder with all kinds of documentation about our
software. When we work on various parts of our code base, we need to make sure
that any documentation there is still up-to-date and accurate, plus reflects
any changes done to the code/functionality/etc.

### Does not contain commits with personal, secret or copyrighted information

Any commits contained in the branch should **not** contain PII.

Any commits contained in the branch should **not** contain any kind of secrets,
private keys, or internal identifiers.

Any commits contained in the branch should **not** contain copyrighted files
or data. Be aware of the license of any item added to the repository which was
not directly made by ourselves. If something is allowed to be distributed but
does not have a purely FOSS license, make sure it's in a `non-free` sub-folder.

Any commits contained in the branch should **not** contain inappropriate things
like erotic material, curse-words, virii, malware, etc.

### Incurred technical debt has a TODO comment and a related issue ticket

From time to time we may introduce (hopefully temporary) technical debt.

When such is the case, we create an issue ticket for it and we make sure that
we have a TODO comment in the code that clearly states what we need to revisit
later on. The comment should include the related issue ticket.

### Can be deployed locally and remotely (CI/CD related setup up-to-date)

A change should maintain a healthy running state. Specifically, the code needs
to be able to run locally through our dev-env, and remotely through our CI/CD
pipeline.

If the change involves any CI/CD pipeline changes, make sure they're implemented
correctly and in such a way that the pipeline still executes as expected.

If the change touches upon any CI/CD pipeline variables, make sure they're set,
either on the GitLab group or project level, or in the GitLab YAML files. These
variables should be documented.

Sometimes we have changes that also require changes in our target environments
which can be local or Kubernetes namespaces. When that is the case, make sure
that those required changes are done in those target environments. For example:
needed ConfigMaps or Secrets, or other types or Kubernetes objects.
