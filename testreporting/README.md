# Test results

This folder contains a wrapper around
[Allure 3](https://github.com/allure-framework/allure3) to create a test report
with all results from our automated tests.

# Rationale

Allure 3 is available as CLI and can be used as such, but lacks an option to
modify the configured readers. That is the primary reason to use the
[AllureReport](https://github.com/allure-framework/allure3/blob/main/packages/core/src/report.ts)
directly and reconfigure the readers.

The readers are reconfigured to group the different tests from the same
component by using the parentSuite attribute of the test result. To ease the
grouping a small file is added that leverages
[yauzl](https://www.npmjs.com/package/yauzl) to read a zipfile directly instead
of manually extracting in a specific dir to group.

# Layout

- [labels.ts](labels.ts) contains the logic to group test results from a
  RawTestResult.
- [generate.ts](generate.ts) contains the wrapper to modify the result while
  reading and the plumbing to generate the report.
- [mapping.ts](mapping.ts) contains the mapping of the filename to
  human-readable name
- [zip.ts](zip.ts) contains the logic to read a zipfile and read the entries as
  ZipResultFile that contains the origin filename.
