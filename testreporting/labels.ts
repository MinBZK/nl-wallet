import type { RawTestResult } from "@allurereport/reader-api"

// Allure has three levels of nesting defined by these properties
const suiteKeys = ["parentSuite", "suite", "subSuite"] as const
type SuiteKey = (typeof suiteKeys)[number]
function isSuiteKey(key: string): key is SuiteKey {
  return (suiteKeys as readonly string[]).indexOf(key) >= 0
}

function setGroupAsParent(suiteNames: Record<SuiteKey, string | undefined>, group: string) {
  // Try to set group as parent by finding a spot and push suites down
  for (let i = 0; i < suiteKeys.length; i++) {
    if (!suiteNames[suiteKeys[i]]) {
      for (let j = i - 1; j >= 0; j--) {
        suiteNames[suiteKeys[j + 1]] = suiteNames[suiteKeys[j]]
      }
      suiteNames[suiteKeys[0]] = group
      return
    }
  }
  // Prefix parentSuite with group
  suiteNames[suiteKeys[0]] = group + " - " + suiteNames[suiteKeys[0]]
}

export function groupTestResult(group: string, result: RawTestResult) {
  if (!result.labels) {
    result.labels = []
  }

  const suiteNames = {} as Record<SuiteKey, string | undefined>
  for (const label of result.labels) {
    if (label.name && isSuiteKey(label.name) && label.value) {
      suiteNames[label.name] = label.value
    }
  }

  setGroupAsParent(suiteNames, group)

  return {
    ...result,
    labels: [
      ...result.labels.filter((l) => l.name && !(l.name in suiteNames)),
      ...suiteKeys.map((key) => ({ name: key, value: suiteNames[key] })).filter((label) => label.value),
    ],
  }
}
