import { promisify } from "node:util"

import { BufferResultFile } from "@allurereport/reader-api"
import { open, type ZipFile, type Options } from "yauzl"

export class ZipResultFile extends BufferResultFile {
  readonly zipFileName: string

  constructor(zipFileName: string, buffer: Uint8Array, fileName: string) {
    super(buffer, fileName)
    this.zipFileName = zipFileName
  }
}

const openZip = promisify<string, Options, ZipFile>(open)

export async function loadFromZip(path: string, callback: (resultFile: ZipResultFile) => Promise<void>) {
  const zipFile = await openZip(path, { autoClose: true, lazyEntries: true })

  return new Promise<void>((resolve, reject) => {
    const openReadStream = promisify(zipFile.openReadStream.bind(zipFile))
    let error = false

    zipFile.on("error", (err) => {
      error = true
      reject(err)
    })

    zipFile.on("close", () => {
      // zipFile auto closes when done reading or on error (when configured with autoClose)
      if (!error) resolve()
    })

    // zipFile will read next entry when calling readEntry (when configured with lazyEntries)
    // no entry event will be emitted yet as readEntry waits on fs.read
    zipFile.readEntry()
    zipFile.on("entry", async (entry) => {
      if (/\/$/.test(entry.fileName)) {
        // Skip directories
        zipFile.readEntry()
      } else {
        try {
          const readStream = await openReadStream(entry)

          // Reading from the async iterator does not seem to work for large files
          const buffers: Buffer[] = []
          readStream.on("data", (chunk) => {
            buffers.push(chunk)
          })
          readStream.on("end", async () => {
            const buffer = Buffer.concat(buffers)
            await callback(new ZipResultFile(path, buffer, entry.fileName))
            zipFile.readEntry()
          })

          readStream.on("err", (err) => {
            error = true
            zipFile.close()
            reject(err)
          })
        } catch (err) {
          error = true
          zipFile.close()
          reject(err)
        }
      }
    })
  })
}
