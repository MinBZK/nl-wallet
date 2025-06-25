# Releases Howto

This guide documents how we do releases, A to Z. The goal is that any team
member can do a release, and knows what steps and which roles are involved.

## Assumptions

* The main git branch is stable and always in a runnable state
* The definitions of done of any merged feature always includes relevant
  documentation we can refer to (config file changes, schema changes, etc)
* The definitions of done of any merged feature always includes any relevant
  unit-, integration and/or end-to-end tests (i.e., features are tested)
* You have `cargo-edit` and `git-filter-repo` installed

## What constitutes a release?

A release is a commit in our Git repository that we tag with a [semver][1]
version tag, and for which we make source code and binaries available.

To do a release, we have a whole bunch of steps we execute. Some of these steps
are manual, some are automated in our CI/CD environment. This guide documents
all those steps, enabling the reader to go through them sequentially, when doing
a release themselves.

## Doing a release

This section of the guide documents all the steps we do when we do a release.

### Step 1: Make sure the release on Jira is green

We use [Jira Releases][2] (also sort-of interchangably called "versions") to
keep track of things we want to have in a release. When you go to our `PVW`
project page on Jira, and click on the "boat" icon on the left, you'll see our
releases page.

For the version you want to release, have a look at the progress bar. When you
click on the release, you'll see all issues in it - essentially, you want all
issues to be in the `Done` state (except possibly for the ones which are related
to doing the release work itself).

If there are still issues `In Progress` or otherwise, have a chit-chat with the
relevant folks with regards to status and discuss with the product owner. When
an issue is still far from done but important for this release, it effectively
blocks the release - you need to wait until the issue is done. Alternatively,
you and/or the product owner might decide that the issue can be moved to the
next release.

When the release is "green" (i.e., all relevant issues are in the `Done` state)
you know that the relevant features and fixes have been merged in the `main`
branch, and you can create a release tag (which we do later on in this guide).

### Step 2: Make sure Figma links are up-to-date in README.md

We have links to Figma in our `README.md` file. We always update these before we
tag the release. Obtain the up-to-date links from the UX team, create an MR and
update the README.

### Step 3: Confirm appropiate tests and reports have been done for this release

We do most testing in an automated fashion using our CI/CD pipeline. There are
however still a few things we confirm. Most notably:

  * Stable main pipeline (CI/CD pipeline for main is green)
  * Automated nightly E2E testsuite succesful
  * Manual E2E testsuite executed and green (there is a metric in our quality
    time instance that our software quality engineer uses for this, you can
    ask him for the link)
  * OSV scanner ran and results accepted (see `osv-scanner` job in pipeline)
  * ZAP scanner ran and results accepted (see ZAP in Quality Time)
  * No blocker or critical Sonar findings (you can check our Sonar instance
    for these findings, ask around for the link if you don't have it)

The manual E2E tests are usually executed by the test automation engineer. We
are working on getting automated ZAP tests as part of the pipeline that will
block on serious issues, and warn on lower priority ones.

The confirmation of the acceptability of the tests and any required reports is
done by our software quality engineer. He will e-mail our shared e-mail account,
the software delivery manager and the product owner with their approval and/or
any remarks or findings.

If there are any findings which can't be accepted or worked around, they will
probably block the release and need to be addressed or otherwise accepted before
further release-steps can occur.

### Step 4: Release approval

The release in Jira is green, manual and automated testing is done and the other
things we confirm in the previous step are all ok or accepted. The product owner
*and* the software delivery manager now need to reply to the e-mail from the
software quality engineer (see previous step) with an approval for release.
The software delivery manager and the product owner CC the shared e-mail
account and the technical team members who are executing the release.

When you have received this CC and it contains the approval from *both* the
product owner and the software delivery manager, you can continue onwards with
the steps to execute the release.

If for some reason the release is not approved, then we as a team need to
address any issues and concerns raised by the PO and/or SDM before further
release-steps can occur.

### Step 5: Freeze git main branch

To avoid someone accidentally or otherwise sniping in another change, you can
temporarily "freeze" the main branch. To do this in GitLab, you can follow
these steps:

  1. Go to the project page, click `Settings`, go to `Repository settings`;
  2. Click `Expand` on `Protected branches` and see the `main` branch there;
  3. Set `Allowed to merge` to `None`;
  4. Set `Allowed to push and merge` to specifically **your** user account;

After doing the above, no one except you can merge or push to the `main branch`.

Don't forget to undo the above when you're done (at least after you've
completed steps 6 and 7).

### Step 6: Set release version and tag

We are now ready to tag the release in our git repository. Before we create the
tag though, we need to make sure that a couple of project files contain the
right version also.

In the `scripts` subdirectory you'll find the `version.sh` release helper. It's
a straightforward tool that assists you in setting the version of our components
and can show you which versions we have already.

Start a new MR and run `version.sh -s RELEASE_VERSION` (where `RELEASE_VERSION`
is something like `v0.2.2`). Run `git status` to see what changed and verify the
sanity of the changes (should be small and easy to comprehend).

Get the MR merged and after merge, run `git tag RELEASE_VERSION` (where
`RELEASE_VERSION` is something like `v0.2.2`). After tagging, push the tag with
`git push --tags`.

### Step 7: Set development version

Immediately start a new MR to set the development version. To do so, run
`version.sh -s DEVELOPMENT_VERSION` (where `DEVELOPMENT_VERSION` is something
like `v0.2.3-dev`). To clarify, when you set a version like `v0.2.3-dev`, you
are indicating that this is the development version of the upcoming `v0.2.3`.
This development version `v0.2.3-dev` will remain set in the component project
files, until we are ready to release, which then leads us to repeat the cycle,
set `v0.2.3` and tag, set the next dev version, etc.

After you've set the development version, merge the MR. It is best if this is
done quickly so people don't accidentally start doing work under the older
version tags.

### Step 8: Collect build artifacts for release

We currently (2024-10-22) collect 4 artifacts from our GitLab CI/CD pipeline:

  * `wallet-sbom_vX.Y.Z_generic.zip`: The software-bill-of-materials for
    this release
  * `wallet-issuance-server_vX.Y.Z_x86_64-linux-glibc.zip`: The wallet issuance
    server for issuers, for glibc-based Linux systems built with Debian
    bookworm.
  * `wallet-issuance-server_vX.Y.Z_x86_64-linux-musl.zip`: The wallet issuance
    server for issuers using statically linked musl.
  * `wallet-verification-server_vX.Y.Z_x86_64-linux-glibc.zip`: The wallet
    verification server for relying parties, for glibc-based Linux systems
    built with Debian bookworm.
  * `wallet-verification-server_vX.Y.Z_x86_64-linux-musl.zip`: The wallet
    verification server for relying parties using statically linked musl.
  * `wallet-web_vX.Y.Z_generic.zip`: The javascript helper library for relying
    parties, to assist with integrating relying party applications with the
    wallet platform

You can collect these artifacts from our GitLab CI/CD pipeline - you need to
go to the relevant job and click on download artifact/zip. You might need to
rename the zip file and/or repackage in the case of `issuance_server` and
`verification_server`.

Currently (2024-11-07) you need to create the sha256sums manually (in the future
we would like to adjust the pipeline that creates the binary artifacts such that
it will create the sha256 hashes also). To create the sha256sum texts, enter the
directory which contains the above mentioned zip files and run:

```shell
for zip in *.zip; do sha256sum $zip > $(echo $zip | sed 's|.zip$|.sha256sum.txt|g'); done
```

When you have these zip files and sha256sum texts, and you made sure they're
named correctly, you are ready to create the release description. We will upload
the zip files and sha256sum texts as artifacts of the release.

Note on other binaries like `wallet_server_migrations` and schema changes in
general: When any of our binaries that use a database backend require schema
changes in the database, we can provide either documentation that instruct how
someone can effect the necessary changes, or a `wallet_server_migrations` utility
and clear instructions with regards to how-to-use. When we as a team decide to
provide a `wallet_server_migrations` binary, make sure that binary is included in
the verification server zip files, with instructions on how to use (i.e., an
end-user must be able to update the schema using the provided utility by
following the additionally supplied read-me or other installation instructions).

Note about obtaining artifacts automatically: Currently the above is manual. We
have an issue on Jira which is about creating an artifact uploader. This utility
is called `uploader.mjs` and can talk to the GitHub releases API and add binary
artifacts and their sha256 sums to a release. The idea is that our pipeline,
when invoked with a CI_COMMIT_TAG environment variable present (i.e., a build
triggered by a `git push --tags` that results in new version tags being pushed)
can invoke this utility and create or update a release with relevant binary
artifacts.

### Step 9: Create a release description

Here is a template for the release description (this is a large body field which
contains markdown describing a given release on GitHub). Make sure you replace
`DAY`, `MONTH`, `YEAR`, `A.B.C`, `X.Y.Z`, `CONDITIONAL_PRE_RELEASE_WARNING`,
`OPTIONAL_RELEASE_STORY`.

```
Release date: DAY of MONTH, YEAR

  * All commits in this release: https://github.com/MinBZK/nl-wallet/compare/vA.B.C...vX.Y.Z
  * Documentation for this release: https://github.com/MinBZK/nl-wallet/blob/vX.Y.Z/documentation/index.md

We have the following artifacts as a part of this release:

  * `wallet-sbom_vX.Y.Z_generic.zip`: The software-bill-of-materials for this release
  * `wallet-verification-server_vX.Y.Z_x86_64-linux-glibc.zip`: The wallet verification server for relying parties, for glibc-based Linux systems
  * `wallet-verification-server_vX.Y.Z_x86_64-linux-musl.zip`: The wallet verification server for relying parties, for musl-libc based Linux systems
  * `wallet-web_vX.Y.Z_generic.zip`: The javascript helper library for relying parties, to assist with integrating relying party applications with the wallet platform

## Notes

**CONDITIONAL_PRE_RELEASE_WARNING**

**OPTIONAL_RELEASE_STORY**

## Changes

See: https://github.com/MinBZK/nl-wallet/blob/vX.Y.Z/documentation/release_notes/vX.Y.Z.md
```

Note on `CONDITIONAL_PRE_RELEASE_WARNING`: When you mark this release as a
pre-release, you need to include the following sentence here: "This is currently
a pre-release; as such it may contain issues we're not aware of and might not
contain full documentation for any introduced changes. A pre-release is intended
for testing purposes and is not production ready, use at your discretion."

Note on `OPTIONAL_RELEASE_STORY`: You can include a paragraph or two here which
details what is special about this release, highlight some features or good to
knows, like config-file-format changes, schema changes, etc.

Note about `CHANGE`: Ensure that this file exists with the proper filename.

After you've created the above release description, save it somewhere so we can
use it in the next step where we're going to create the actual GitHub release.

### Step 10: Create GitHub release from tag

When you visit our GitHub page, you should now see all the additional commits
and the version tag you've previously set to indicate the release. This version
tag will point to the head of the repository and will not have an associated
release yet.

  1. Go to the *Releases* page and click on the *Draft a new release* icon.
     You'll be presented with a release creation page;
  2. Click on *Choose a tag* and select the version you want to create a release
     for (normally the latest one for which you did all the previous steps);
  3. As title, write: `Wallet X.Y.Z`, where `X.Y.Z` is the version number you're
     releasing (i.e., should match tag without the `v` prefix);
  4. Add the previously collected/created zip files and sha256 text files;
  5. Insert the previously created release description markdown in the body text
     of this release;
  6. Enable the *Set as a pre-release* flag;
  7. Click on *Publish release*;

Note on setting the pre-release flag: We always set this flag when doing an
initial release. A versioned release like we've just created is also going
through a larger scale testing and deployment phase through the operations
team. When they have finished their testing phases and obtained their relevant
approvals, a go-ahead could be given to unset the pre-release flag. This is an
asynchronous process and never blocks the release itself (i.e., don't wait, just
leave the release marked as pre-release until someone comes around later on to
tell you/us that a specific release can have the pre-release flag removed)

Note on removing a pre-release flag: This usually only happens when the software
is fully vetted by the operations team and deemed ready to run on our production
backend. When the flag is removed (and so becomes a considered-stable release),
don't forget to remove the `CONDITIONAL_PRE_RELEASE_WARNING` paragraph from the
release description.

## References

1. [Semantic Versioning](https://www.semver.org/)
2. [Agile Versions](https://www.atlassian.com/agile/tutorials/versions/)
