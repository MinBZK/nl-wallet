export function createAbsoluteUrl(baseUrl: string, href: string, pathname: string) {
  let base = href

  /* Strip document from path, if any */
  if (!pathname.endsWith("/")) {
    const segment = pathname.split("/").pop()
    if (segment) {
      base = href.replace(segment, "")
    }
  }

  /* Make baseUrl absolute. If it already is, the second argument is ignored. */
  return new URL(baseUrl, base).toString()
}
