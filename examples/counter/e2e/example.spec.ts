import { test, expect, Page } from "@playwright/test";
import i18nEn from "#locales/en.json";
import i18nFr from "#locales/fr.json";
import { fail_windows_webkit, TestArgs, BrowserName } from "../../utils";

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

test.describe("when locale is the default locale (en-GB)", () => {
  test.skip(fail_windows_webkit, "webkit does not support wasm on windows");

  test("check counter", ({ page }) => check_counter(page, EN_LOCALE));
  test("check lang switch", ({ page, browserName }) =>
    check_lang_switch(page, browserName, FR_LOCALE));
  test("check state keeping", ({ page }) =>
    check_state_keeping(page, EN_LOCALE, FR_LOCALE));
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.skip(fail_windows_webkit, "webkit does not support wasm on windows");

  test.use({
    locale: "fr",
  });

  test("check counter", ({ page }) => check_counter(page, FR_LOCALE));
  test("check lang switch", ({ page, browserName }) =>
    check_lang_switch(page, browserName, EN_LOCALE));
  test("check state keeping", ({ page }) =>
    check_state_keeping(page, FR_LOCALE, EN_LOCALE));
});

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
  await page.locator(INC_BUTTON_XPATH).click();
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    locale.locale.click_count.replace("{{ count }}", "1")
  );
  await page.locator(INC_BUTTON_XPATH).click();
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    locale.locale.click_count.replace("{{ count }}", "2")
  );
  await page.locator(INC_BUTTON_XPATH).click();
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    locale.locale.click_count.replace("{{ count }}", "3")
  );
}

async function check_lang_switch(
  page: Page,
  browserName: BrowserName,
  locale: LocaleArg
) {
  await page.goto("/");

  await page.locator(LNG_BUTTON_XPATH).click();

  await check_counter(page, locale, false);

  // FIXME: cookies aren't working on webkit for some reason ?
  // if (browserName == "webkit") {
  //   return;
  // }

  await page.reload();
  // check if locale persist
  await check_counter(page, locale, false);
}

async function check_state_keeping(
  page: Page,
  current_locale: LocaleArg,
  target_locale: LocaleArg
) {
  await page.goto("/");

  await page.locator(INC_BUTTON_XPATH).click({ clickCount: 3 });

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    current_locale.locale.click_count.replace("{{ count }}", "3")
  );

  await page.locator(LNG_BUTTON_XPATH).click();

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    target_locale.locale.click_count.replace("{{ count }}", "3")
  );

  await page.locator(INC_BUTTON_XPATH).click();
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    target_locale.locale.click_count.replace("{{ count }}", "4")
  );
}
