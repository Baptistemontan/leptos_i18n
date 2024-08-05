import { test as base, expect, Page } from "@playwright/test";
import i18nEn from "#locales/en.json";
import i18nFr from "#locales/fr.json";
import { fail_windows_webkit, createI18nFixture } from "../../../utils";

const LNG_BUTTON_XPATH = "xpath=//html/body/button[1]";
const TITLE_XPATH = "xpath=//html/body/h1";
const INC_BUTTON_XPATH = "xpath=//html/body/button[2]";
const COUNTER_XPATH = "xpath=//html/body/p";

const test = base.extend(
  createI18nFixture({
    default_locale: "en",
    locales: {
      en: i18nEn,
      fr: i18nFr,
    },
  })
);

type I18n = Parameters<Parameters<typeof test>[2]>[0]["i18n"];

async function switch_lang(i18n: I18n) {
  if (i18n.locale == "en") {
    await i18n.set_locale("fr");
  } else {
    await i18n.set_locale("en");
  }
}

test.skip(fail_windows_webkit, "webkit does not support wasm on windows");

test.beforeEach(async ({ i18n, page }) => {
  i18n.on_locale_change(async () => {
    await page.locator(LNG_BUTTON_XPATH).click();
  });
});

test.afterEach(async ({ context }) => {
  await context.clearCookies();
});

test.describe("when locale is the default locale (en-GB)", () => {
  test("check counter", ({ page, i18n }) => check_counter(page, i18n));
  test("check lang switch", ({ page, i18n }) => check_lang_switch(page, i18n));
  test("check state keeping", ({ page, i18n }) =>
    check_state_keeping(page, i18n));
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.use({
    locale: "fr-FR",
  });

  test("check counter", ({ page, i18n }) => check_counter(page, i18n));
  test("check lang switch", ({ page, i18n }) => check_lang_switch(page, i18n));
  test("check state keeping", ({ page, i18n }) =>
    check_state_keeping(page, i18n));
});

async function check_counter(
  page: Page,
  i18n: I18n,
  load_page: boolean = true
) {
  if (load_page) {
    await page.goto("/");
  }

  await expect(page.locator(TITLE_XPATH)).toHaveText(i18n.t("hello_world"));
  await expect(page.locator(LNG_BUTTON_XPATH)).toHaveText(
    i18n.t("click_to_change_lang")
  );
  await expect(page.locator(INC_BUTTON_XPATH)).toHaveText(
    i18n.t("click_to_inc")
  );
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 0 })
  );

  await page.locator(INC_BUTTON_XPATH).click();
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 1 })
  );
  await page.locator(INC_BUTTON_XPATH).click();
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 2 })
  );
  await page.locator(INC_BUTTON_XPATH).click();
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 3 })
  );
}

async function check_lang_switch(page: Page, i18n: I18n) {
  await page.goto("/");

  await switch_lang(i18n);

  await check_counter(page, i18n, false);
  // check if locale persist
  await page.reload();
  await check_counter(page, i18n, false);
}

async function check_state_keeping(page: Page, i18n: I18n) {
  await page.goto("/");

  await page.locator(INC_BUTTON_XPATH).click({ clickCount: 3 });

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 3 })
  );

  await switch_lang(i18n);

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 3 })
  );

  await page.locator(INC_BUTTON_XPATH).click();
  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 4 })
  );
}
