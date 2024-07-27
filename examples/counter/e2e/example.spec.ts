import { test, expect, Page } from "@playwright/test";
import i18nEn from "#locales/en.json";
import i18nFr from "#locales/fr.json";
import * as os from "os";

type TestArgs = Parameters<Parameters<typeof test>[2]>[0]; // wonky
type BrowserContext = TestArgs["context"];
const COOKIE_PREFERED_LANG = "i18n_pref_locale";
const WIN = os.platform() == "win32";

const LNG_BUTTON_XPATH = "xpath=//html/body/button[1]";
const INC_BUTTON_XPATH = "xpath=//html/body/button[2]";
const COUNTER_XPATH = "xpath=//html/body/p";

type Locale = typeof i18nEn;

interface LocaleArg {
  locale: Locale;
  id: string;
}

const EN_LOCALE: LocaleArg = {
  locale: i18nEn,
  id: "en",
};

const FR_LOCALE: LocaleArg = {
  locale: i18nFr,
  id: "fr",
};

function fail_windows_webkit({ browserName }: TestArgs): boolean {
  const WEBKIT = browserName === "webkit";
  return WEBKIT && WIN;
}

test.describe("when locale is the default locale (en-GB)", () => {
  test.fail(fail_windows_webkit, "webkit does not support wasm on windows");

  test("simple test", ({ page, context }) =>
    check(page, context, EN_LOCALE, FR_LOCALE));
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.fail(fail_windows_webkit, "webkit does not support wasm on windows");

  test.use({
    locale: "fr",
  });

  test("simple test", ({ page, context }) =>
    check(page, context, FR_LOCALE, EN_LOCALE));
});

async function check_cookie(
  context: BrowserContext,
  name: string,
  expected_value: string
) {
  const cookies = await context.cookies();
  await expect(cookies.find((c) => c.name == name)?.value).toBe(expected_value);
}

async function check(
  page: Page,
  context: BrowserContext,
  first_locale: LocaleArg,
  second_locale: LocaleArg
) {
  await page.goto("/");

  const lngButton = page.locator(LNG_BUTTON_XPATH);

  await expect(lngButton).toHaveText(first_locale.locale.click_to_change_lang);

  const incButton = page.locator(INC_BUTTON_XPATH);

  await expect(incButton).toHaveText(first_locale.locale.click_to_inc);

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    first_locale.locale.click_count.replace("{{ count }}", "0")
  );

  await incButton.click();

  await check_cookie(context, COOKIE_PREFERED_LANG, first_locale.id);

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    first_locale.locale.click_count.replace("{{ count }}", "1")
  );

  // switch locales

  await lngButton.click();

  await expect(page.locator(LNG_BUTTON_XPATH)).toHaveText(
    second_locale.locale.click_to_change_lang
  );
  await expect(page.locator(INC_BUTTON_XPATH)).toHaveText(
    second_locale.locale.click_to_inc
  );
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    second_locale.locale.click_count.replace("{{ count }}", "1")
  );

  await check_cookie(context, COOKIE_PREFERED_LANG, second_locale.id);
}
