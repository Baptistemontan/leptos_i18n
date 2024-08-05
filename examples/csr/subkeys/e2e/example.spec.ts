import { test as base, expect, Page } from "@playwright/test";
import i18nEn from "#locales/en.json";
import i18nFr from "#locales/fr.json";
import { fail_windows_webkit, createI18nFixture } from "../../../utils";

const LNG_BUTTON_XPATH = "xpath=//html/body/button[1]";
const SUBKEY_XPATH = "xpath=//html/body/p[1]";
const SUBKEY_SUBKEY_XPATH = "xpath=//html/body/p[2]";

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
  test("check values", ({ page, i18n }) => check_values(page, i18n));
  test("check lang switch", ({ page, i18n }) => check_lang_switch(page, i18n));
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.use({
    locale: "fr-FR",
  });

  test("check values", ({ page, i18n }) => check_values(page, i18n));
  test("check lang switch", ({ page, i18n }) => check_lang_switch(page, i18n));
});

async function check_values(page: Page, i18n: I18n, load_page: boolean = true) {
  if (load_page) {
    await page.goto("/");
  }

  await expect(page.locator(LNG_BUTTON_XPATH)).toHaveText(
    i18n.t("click_to_change_lang")
  );

  await expect(page.locator(SUBKEY_XPATH)).toHaveText(i18n.t("subkeys.subkey"));
  await expect(page.locator(SUBKEY_SUBKEY_XPATH)).toHaveText(
    i18n.t("subkeys.subsubkeys.subsubkey")
  );
}

async function check_lang_switch(page: Page, i18n: I18n) {
  await page.goto("/");

  await switch_lang(i18n);

  await check_values(page, i18n, false);
  // check if locale persist
  await page.reload();
  await check_values(page, i18n, false);
}
