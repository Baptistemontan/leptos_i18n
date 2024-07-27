import { test, expect } from "@playwright/test";
import i18nEn from "#locales/en.json";
import i18nFr from "#locales/fr.json";
import * as os from "os";

type TestArgs = Parameters<Parameters<typeof test>[2]>[0]; // wonky
type BrowserContext = TestArgs["context"];
const COOKIE_PREFERED_LANG = "i18n_pref_locale";
const WIN = os.platform() == "win32";

function fail_windows_webkit({ browserName }: TestArgs): boolean {
  const WEBKIT = browserName === "webkit";
  return WEBKIT && WIN;
}

test.describe("when locale is the default locale (en-GB)", () => {
  test.fail(fail_windows_webkit, "webkit does not support wasm on windows");

  test("simple test", check_english);
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.fail(fail_windows_webkit, "webkit does not support wasm on windows");

  test.use({
    locale: "fr",
  });

  test("simple test", check_french);
});

async function check_cookie(
  context: BrowserContext,
  name: string,
  expected_value: string
) {
  const cookies = await context.cookies();
  await expect(cookies.find((c) => c.name == name)?.value).toBe(expected_value);
}

async function check_english({ page, context }: TestArgs) {
  await page.goto("/");

  const lngButton = page.getByRole("button", {
    name: i18nEn.click_to_change_lang,
  });

  await expect(lngButton).toBeVisible();

  const incButton = page.getByRole("button", {
    name: i18nEn.click_to_inc,
  });

  await expect(incButton).toBeVisible();

  await expect(
    page.getByText(i18nEn.click_count.replace("{{ count }}", "0"))
  ).toBeVisible();

  await incButton.click();

  await check_cookie(context, COOKIE_PREFERED_LANG, "en");

  await expect(
    page.getByText(i18nEn.click_count.replace("{{ count }}", "1"))
  ).toBeVisible();

  // switch to french

  await lngButton.click();

  await expect(
    page.getByRole("button", {
      name: i18nFr.click_to_change_lang,
    })
  ).toBeVisible();
  await expect(
    page.getByRole("button", {
      name: i18nFr.click_to_inc,
    })
  ).toBeVisible();
  await expect(
    page.getByText(i18nFr.click_count.replace("{{ count }}", "1"))
  ).toBeVisible();

  await check_cookie(context, COOKIE_PREFERED_LANG, "fr");
}

async function check_french({ page, context }: TestArgs) {
  await page.goto("/");

  const lngButton = page.getByRole("button", {
    name: i18nFr.click_to_change_lang,
  });

  await expect(lngButton).toBeVisible();

  const incButton = page.getByRole("button", {
    name: i18nFr.click_to_inc,
  });

  await expect(incButton).toBeVisible();

  await expect(
    page.getByText(i18nFr.click_count.replace("{{ count }}", "0"))
  ).toBeVisible();

  await incButton.click();

  await check_cookie(context, COOKIE_PREFERED_LANG, "fr");

  await expect(
    page.getByText(i18nFr.click_count.replace("{{ count }}", "1"))
  ).toBeVisible();

  // switch to english
  await lngButton.click();

  await expect(
    page.getByRole("button", {
      name: i18nEn.click_to_change_lang,
    })
  ).toBeVisible();
  await expect(
    page.getByRole("button", {
      name: i18nEn.click_to_inc,
    })
  ).toBeVisible();
  await expect(
    page.getByText(i18nEn.click_count.replace("{{ count }}", "1"))
  ).toBeVisible();

  await check_cookie(context, COOKIE_PREFERED_LANG, "en");
}
