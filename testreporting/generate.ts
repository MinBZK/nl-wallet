import { AllureReport, readConfig } from "@allurereport/core"
import { PathResultFile } from "@allurereport/reader-api"
import { allure2, attachments, junitXml } from "@allurereport/reader"

import { GroupedReader } from "./reader.ts"
import { loadFromZip } from "./zip.ts"

/*
 * Read config via allurerc.mjs as Allure CLI and wrap readers
 */
const config = await readConfig()
config.readers = [allure2, junitXml, attachments].map((r) => new GroupedReader(r))

/*
 * Generate report from program arguments
 */
const report = new AllureReport(config)
await report.start()

for (const arg of process.argv.slice(2)) {
  console.log(`Processing ${arg}`)
  const resultFile = new PathResultFile(arg)

  if (resultFile.getContentType() == "application/zip") {
    await loadFromZip(arg, async (resultEntry) => {
      await report.readResult(resultEntry)
    })
  } else {
    await report.readResult(resultFile)
  }
}

await report.done()
