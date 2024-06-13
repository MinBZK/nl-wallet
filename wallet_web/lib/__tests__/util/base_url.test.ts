import { createAbsoluteUrl } from "@/util/base_url"
import { describe, expect, test } from "vitest"

describe("createAbsoluteUrl", () => {
  test.each([
    ["", "http://localhost:3004/", "/", "http://localhost:3004/"],
    [".", "http://localhost:3004/", "/", "http://localhost:3004/"],
    ["./", "http://localhost:3004/", "/", "http://localhost:3004/"],
    ["..", "http://localhost:3004/path1/", "/path1/", "http://localhost:3004/"],
    ["../", "http://localhost:3004/path1/", "/path1/", "http://localhost:3004/"],
    ["../path2", "http://localhost:3004/path1/", "/path1/", "http://localhost:3004/path2"],
    ["", "https://localhost/abcd123/index.hml", "/abcd123/index.hml", "https://localhost/abcd123/"],
    ["", "https://localhost/abcd123/", "/abcd123/", "https://localhost/abcd123/"],
    ["../", "https://localhost/abcd123/foo/", "/abcd123/foo/", "https://localhost/abcd123/"],
    [
      "../path2",
      "http://localhost:3004/path1/index.html",
      "/path1/index.html",
      "http://localhost:3004/path2",
    ],
    [
      "./path2",
      "http://localhost:3004/path1/path1a/path1b/index.html",
      "/path1/path1a/path1b/index.html",
      "http://localhost:3004/path1/path1a/path1b/path2",
    ],
    [
      "/path2",
      "http://localhost:3004/path1/path1a/path1b/index.html",
      "/path1/path1a/path1b/index.html",
      "http://localhost:3004/path2",
    ],
    [
      "/path2",
      "http://localhost:3004/path1/path1a/path1b/",
      "/path1/path1a/path1b/",
      "http://localhost:3004/path2",
    ],
    [
      "http://192.168.1.1:3003/path2",
      "http://localhost:3004/path1/",
      "/path1/",
      "http://192.168.1.1:3003/path2",
    ],
    [
      "http://192.168.1.1:3003/path2/",
      "http://localhost:3004/path1/",
      "/path1/",
      "http://192.168.1.1:3003/path2/",
    ],
  ])("should detect desktop for useragent: %s", (base_url, href, path, expected) => {
    expect(createAbsoluteUrl(base_url, href, path).toString()).toEqual(expected)
  })
})
