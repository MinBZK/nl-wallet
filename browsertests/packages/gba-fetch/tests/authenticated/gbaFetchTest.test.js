import { test, expect } from "@playwright/test"
import { GbaFetchPage } from "../../pages/gbaFetchPage.js"

test.describe("GBA Fetch BRP preloading", () => {
  let gbaFetchPage

  test.beforeEach(async ({ page, baseURL }) => {
    gbaFetchPage = new GbaFetchPage(page)
    await page.goto(baseURL)
  })

  test("The system displays a webpage with two forms (1) load data and (2) clear all data.", async () => {
    expect(await gbaFetchPage.getPageTitle()).toBe("GBA-V preloading for NL Wallet")
    expect(await gbaFetchPage.getStatusInfoLabel()).toBe("Number of preloaded BSNs: ")
    expect(await gbaFetchPage.getStatusInfoPreloadedCount()).toMatch(/^\d+$/)

    await expect(gbaFetchPage.getBsnInput()).toBeVisible()
    await expect(gbaFetchPage.getRepeatBsnInput()).toBeVisible()
    await expect(gbaFetchPage.getPreloadButton()).toBeVisible()

    await expect(gbaFetchPage.getClearDataLabel()).toBeVisible()
    await expect(gbaFetchPage.getClearDataInput()).toBeVisible()
    await expect(gbaFetchPage.getClearDataButton()).toBeVisible()
  })

  test("The load data form has two password inputs (i.e. not showing the value) that take a BSN and a submit button.", async () => {
    await expect(gbaFetchPage.getBsnInput()).toHaveAttribute("type", "password")
    await expect(gbaFetchPage.getBsnInput()).toHaveAttribute("type", "password")
    await expect(gbaFetchPage.getPreloadButton()).toHaveAttribute("type", "submit")
  })

  test("On submitting the load data form, the system validates that both BSN entries match", async () => {
    await gbaFetchPage.enterBsn("999994761")
    await gbaFetchPage.repeatBsn("999994797")
    await gbaFetchPage.preload()
    expect(await gbaFetchPage.getResult()).toBe("BSNs do not match")
  })

  test("On submitting the load data form, the system validates that The BSN entries are a valid BSN (8-9 digits, matching elfproef[1])", async () => {
    await gbaFetchPage.enterBsn("99999476b")
    await gbaFetchPage.repeatBsn("99999476b")
    await gbaFetchPage.preload()
    expect(await gbaFetchPage.getBsnValidationMessage()).toContain("the requested format")

    await gbaFetchPage.enterBsn("9999947")
    await gbaFetchPage.repeatBsn("9999947")
    await gbaFetchPage.preload()
    expect(await gbaFetchPage.getBsnValidationMessage()).toContain("the requested format")

    await gbaFetchPage.enterBsn("99999999")
    await gbaFetchPage.repeatBsn("99999999")
    await gbaFetchPage.preload()
    expect(await gbaFetchPage.getResult()).toBe("Bsn failed the predicate test.")
  })

  test("If data belonging to that BSN was already present, the system simply overwrites the data.", async () => {
    await gbaFetchPage.enterBsn("999994906")
    await gbaFetchPage.repeatBsn("999994906")
    await gbaFetchPage.preload()
    await gbaFetchPage.goBack()
    let preloadedCount = await gbaFetchPage.getStatusInfoPreloadedCount()
    await gbaFetchPage.enterBsn("999994906")
    await gbaFetchPage.repeatBsn("999994906")
    await gbaFetchPage.preload()
    await gbaFetchPage.goBack()
    expect(await gbaFetchPage.getStatusInfoPreloadedCount()).toBe(preloadedCount)
  })

  test("On succesful storage, the system shows a confirmation, including the number of currently loaded datasets.", async () => {
    await gbaFetchPage.enterBsn("999994906")
    await gbaFetchPage.repeatBsn("999994906")
    await gbaFetchPage.preload()
    expect(await gbaFetchPage.getResult()).toBe("Successfully preloaded")
    await gbaFetchPage.goBack()
    expect(Number(await gbaFetchPage.getStatusInfoPreloadedCount())).toBeGreaterThan(0)
  })

  test('The clear all data form takes one input: the user must enter the string "clear all data" to avoid accidental deletion.', async () => {
    expect(await gbaFetchPage.getClearDataLabelText()).toBe(
      'Enter the text "clear all data" for confirmation: ',
    )
    await gbaFetchPage.enterClearDataText("clear")
    await gbaFetchPage.clearData()
    expect(await gbaFetchPage.getResult()).toBe("Confirmation text is not correct")
  })

  test("On submitting the clear all data form with the valid string, the system clears all data in the cache and reports the new number of stored data sets (must be 0).", async () => {
    await gbaFetchPage.enterBsn("999994906")
    await gbaFetchPage.repeatBsn("999994906")
    await gbaFetchPage.preload()
    await gbaFetchPage.goBack()
    expect(await gbaFetchPage.getStatusInfoPreloadedCount()).toBe("1")
    expect(await gbaFetchPage.getClearDataLabelText()).toBe(
      'Enter the text "clear all data" for confirmation: ',
    )
    await gbaFetchPage.enterClearDataText("clear all data")
    await gbaFetchPage.clearData()
    expect(await gbaFetchPage.getResult()).toContain("Successfully cleared ")
    await gbaFetchPage.goBack()
    expect(await gbaFetchPage.getStatusInfoPreloadedCount()).toBe("0")
  })
})
