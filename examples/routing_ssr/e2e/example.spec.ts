import { test as base, expect, Page } from "@playwright/test";
import i18nEn from "#locales/en.json";
import i18nFr from "#locales/fr.json";
import { fail_windows_webkit, createI18nFixture } from "../../utils";

const LNG_BUTTON_XPATH = "xpath=//html/body/button";

const TITLE_XPATH = "xpath=//html/body/h1";
const COUNTER_ANCHOR_XPATH = "xpath=//html/body/a";

const COUNTER_XPATH = "xpath=//html/body/div/p";
const INC_BUTTON_XPATH = "xpath=//html/body/div/button";
const HOME_ANCHOR_XPATH = "xpath=//html/body/div/a";

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
type Locale = Parameters<I18n["set_locale"]>[0];

async function switch_lang(i18n: I18n): Promise<Locale> {
  if (i18n.locale == "en") {
    await i18n.set_locale("fr");
    return "fr";
  } else {
    await i18n.set_locale("en");
    return "en";
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
  test("main check", ({ page, i18n }) => main_check(page, i18n));
  test("history check", ({ page, i18n }) => history_check(page, i18n));
  test("counter check", ({ page, i18n }) => counter_check(page, i18n));
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.use({
    locale: "fr-FR",
  });

  test("main check", ({ page, i18n }) => main_check(page, i18n));
  test("history check", ({ page, i18n }) => history_check(page, i18n));
  test("counter check", ({ page, i18n }) => counter_check(page, i18n));
});

async function main_check(page: Page, i18n: I18n) {
  await page.goto("/");

  await expect(page).toHaveURL(i18n.get_url());

  await expect(page.locator(TITLE_XPATH)).toHaveText(i18n.t("hello_world"));

  await expect(page.locator(LNG_BUTTON_XPATH)).toHaveText(
    i18n.t("click_to_change_lang")
  );

  await expect(page.locator(COUNTER_ANCHOR_XPATH)).toHaveText(
    i18n.t("go_counter")
  );

  await page.locator(COUNTER_ANCHOR_XPATH).click();

  await expect(page).toHaveURL(i18n.get_url("counter"));

  await expect(page.locator(LNG_BUTTON_XPATH)).toHaveText(
    i18n.t("click_to_change_lang")
  );

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 0 })
  );

  await expect(page.locator(INC_BUTTON_XPATH)).toHaveText(
    i18n.t("click_to_inc")
  );

  await expect(page.locator(HOME_ANCHOR_XPATH)).toHaveText(i18n.t("go_home"));

  await page.locator(INC_BUTTON_XPATH).click({ clickCount: 3 });

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 3 })
  );

  await page.locator(HOME_ANCHOR_XPATH).click();

  await expect(page).toHaveURL(i18n.get_url());
}

async function history_check(page: Page, i18n: I18n) {
  await page.goto("/");

  await expect(page.locator(TITLE_XPATH)).toHaveText(i18n.t("hello_world"));

  await expect(page).toHaveURL(i18n.get_url());
  const prev_locale = i18n.locale;

  const next_locale = await switch_lang(i18n);
  await expect(page.locator(TITLE_XPATH)).toHaveText(i18n.t("hello_world"));
  await expect(page).toHaveURL(i18n.get_url());

  await page.goBack();
  i18n.set_locale_untracked(prev_locale);
  await expect(page.locator(TITLE_XPATH)).toHaveText(i18n.t("hello_world"));
  await expect(page).toHaveURL(i18n.get_url());

  await page.goForward();
  i18n.set_locale_untracked(next_locale);
  await expect(page.locator(TITLE_XPATH)).toHaveText(i18n.t("hello_world"));
  await expect(page).toHaveURL(i18n.get_url());
}

async function counter_check(page: Page, i18n: I18n) {
  await page.goto("/counter");

  await expect(page).toHaveURL(i18n.get_url("counter"));

  await expect(page.locator(LNG_BUTTON_XPATH)).toHaveText(
    i18n.t("click_to_change_lang")
  );

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 0 })
  );

  await page.locator(INC_BUTTON_XPATH).click({ clickCount: 3 });

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 3 })
  );

  await switch_lang(i18n);

  await expect(page).toHaveURL(i18n.get_url("counter"));

  await expect(page.locator(COUNTER_XPATH)).toHaveText(
    i18n.t("click_count", { count: 3 })
  );
}
