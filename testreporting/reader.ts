import { basename } from "node:path"

import type { ResultFile } from "@allurereport/plugin-api"
import type { RawTestResult, ReaderContext, ResultsReader, ResultsVisitor } from "@allurereport/reader-api"

import { groupTestResult } from "./labels.ts"
import { fileNameToGroup } from "./mapping.ts"
import { ZipResultFile } from "./zip.ts"

// Reader that wraps ResultsReader and uses groupTestResult to group the results from the origin
export class GroupedReader implements ResultsReader {
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
