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
  test.skip(fail_windows_webkit, "webkit does not support wasm on windows");

  test("check counter", ({ page }) => check_counter(page, EN_LOCALE));
  test("check lang switch", ({ page }) => check_lang_switch(page, FR_LOCALE));
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.skip(fail_windows_webkit, "webkit does not support wasm on windows");

  test.use({
    locale: "fr",
  });

  test("check counter", ({ page }) => check_counter(page, FR_LOCALE));
  test("check lang switch", ({ page }) => check_lang_switch(page, EN_LOCALE));
});

async function check_cookie(
  context: BrowserContext,
  name: string,
  expected_value: string
) {
  const cookies = await context.cookies();
  await expect(cookies.find((c) => c.name == name)?.value).toBe(expected_value);
}

async function check_counter(
  page: Page,
  locale: LocaleArg,
  load_page: boolean = true
) {
  if (load_page) {
    await page.goto("/");
  }

  await expect(page.locator(LNG_BUTTON_XPATH)).toHaveText(
    locale.locale.click_to_change_lang
  );
  await expect(page.locator(INC_BUTTON_XPATH)).toHaveText(
    locale.locale.click_to_inc
  );
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    locale.locale.click_count.replace("{{ count }}", "0")
  );

  for (let i = 1; i < 3; i++) {
    await page.locator(INC_BUTTON_XPATH).click();

    await expect(page.locator(COUNTER_XPATH)).toHaveText(
      locale.locale.click_count.replace("{{ count }}", i.toString())
    );
  }
}

async function check_lang_switch(page: Page, locale: LocaleArg) {
  await page.goto("/");

  await page.locator(LNG_BUTTON_XPATH).click();

  await check_counter(page, locale, false);

  await page.reload();
  // check if locale persist
  await check_counter(page, locale, false);
}
