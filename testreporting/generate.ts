import { basename } from "node:path"

import { AllureReport, readConfig } from "@allurereport/core"
import type { ResultFile } from "@allurereport/plugin-api"
import { PathResultFile } from "@allurereport/reader-api"
import type {
  RawTestResult,
  ReaderContext,
  ResultsReader,
  ResultsVisitor,
} from "@allurereport/reader-api"
import { allure2, attachments, junitXml } from "@allurereport/reader"

import { groupTestResult } from "./labels.ts"
import { loadFromZip, ZipResultFile } from "./zip.ts"
import { fileNameToGroup } from "./mapping.ts"

// Reader that wraps ResultsReader and uses groupTestResult to group the results from the origin
class GroupedReader implements ResultsReader {
  readonly #reader: ResultsReader

  constructor(reader: ResultsReader) {
    this.#reader = reader
  }

  read(visitor: ResultsVisitor, data: ResultFile) {
    const originFileName = this.#originFileName(data)
    if (!originFileName) {
      return this.#reader.read(visitor, data)
    }
    const group = fileNameToGroup(originFileName)

    const wrappedVisitor = {
      visitTestResult: (result: RawTestResult, context: ReaderContext) =>
        visitor.visitTestResult(groupTestResult(group, result), context),
      visitTestFixtureResult: visitor.visitTestFixtureResult.bind(visitor),
      visitAttachmentFile: visitor.visitAttachmentFile.bind(visitor),
      visitMetadata: visitor.visitMetadata.bind(visitor),
    }
    return this.#reader.read(wrappedVisitor, data)
  }

  readerId(): string {
    return this.#reader.readerId()
  }

  #originFileName(resultFile: ResultFile) {
    if (this.#reader.readerId() == "junit") {
      return resultFile.getOriginalFileName()
    } else if (resultFile instanceof ZipResultFile) {
      return basename(resultFile.zipFileName)
    }
  }
}

// Read config via allurerc.mjs as Allure CLI and wrap readers
const config = await readConfig()
config.readers = [allure2, junitXml, attachments].map((r) => new GroupedReader(r))

// Generate report from program arguments
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
