import { test as base, expect, Page } from "@playwright/test";
import i18nEn from "#locales/en.json";
import i18nFr from "#locales/fr.json";
import {
  fail_windows_webkit,
  createI18nFixture,
  TDFunc,
  GetL,
  MaybeTd,
} from "../../../utils";

type XPaths = {
  main: string;
  opposite: string;
  cookie: string;
  langAttr: string;
};

const LNG_BUTTON_XPATH: XPaths = {
  main: "xpath=//html/body/button[2]",
  opposite: "xpath=//html/body/button[4]",
  cookie: "xpath=//html/body/button[6]",
  langAttr: "xpath=//html/body/div[1]/button[2]",
};

const INC_BUTTON_XPATH: XPaths = {
  main: "xpath=//html/body/button[1]",
  opposite: "xpath=//html/body/button[3]",
  cookie: "xpath=//html/body/button[5]",
  langAttr: "xpath=//html/body/div[1]/button[1]",
};
const COUNTER_XPATH: XPaths = {
  main: "xpath=//html/body/p[1]",
  opposite: "xpath=//html/body/p[2]",
  cookie: "xpath=//html/body/p[3]",
  langAttr: "xpath=//html/body/div[1]/p",
};
const LOCALES_XPATH: XPaths = {
  main: "xpath=//html/body/h1[1]",
  opposite: "xpath=//html/body/h1[2]",
  cookie: "xpath=//html/body/h1[3]",
  langAttr: "xpath=//html/body/div[1]/h1",
};
const TITLES_XPATH: XPaths = {
  main: "xpath=//html/body/h2[1]",
  opposite: "xpath=//html/body/h2[2]",
  cookie: "xpath=//html/body/h2[3]",
  langAttr: "xpath=//html/body/h2[4]",
};

const test = base.extend(
  createI18nFixture({
    default_locale: "en",
    locales: {
      en: i18nEn,
      fr: i18nFr,
    },
  })
);

type XPathsKey = keyof XPaths;

type I18n = Parameters<Parameters<typeof test>[2]>[0]["i18n"];
type TFunc = I18n["t"];

type L = GetL<I18n>;

type Locale = keyof L;

async function switch_lang(i18n: I18n) {
  if (i18n.locale == "en") {
    await i18n.set_locale("fr");
  } else {
    await i18n.set_locale("en");
  }
}

function make_t(
  i18n: I18n
): (locale: Locale | undefined, ...args: Parameters<TFunc>) => string {
  return (locale, ...args) =>
    locale ? i18n.td(locale, ...args) : i18n.t(...args);
}

test.skip(fail_windows_webkit, "webkit does not support wasm on windows");

test.beforeEach(async ({ i18n, page }) => {
  i18n.on_locale_change(async () => {
    await page.locator(LNG_BUTTON_XPATH.main).click();
  });
});

test.afterEach(async ({ context }) => {
  await context.clearCookies();
});

test.describe("when locale is the default locale (en-GB)", () => {
  test("check main counter", ({ page, i18n, maybe_td }) =>
    check_counter(page, i18n, maybe_td, "main"));
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.use({
    locale: "fr-FR",
  });

  test("check main counter", ({ page, i18n, maybe_td }) =>
    check_counter(page, i18n, maybe_td, "main"));
});

async function check_counter(
  page: Page,
  i18n: I18n,
  maybe_td: MaybeTd<L>,
  xpaths: keyof XPaths,
  locale?: Locale,
  load_page: boolean = true
) {
  if (load_page) {
    await page.goto("/");
  }

  await expect(page.locator(LNG_BUTTON_XPATH[xpaths])).toHaveText(
    maybe_td(locale, "click_to_change_lang")
  );
  await expect(page.locator(INC_BUTTON_XPATH[xpaths])).toHaveText(
    maybe_td(locale, "click_to_inc")
  );

  await expect(page.locator(COUNTER_XPATH[xpaths])).toHaveText(
    maybe_td(locale, "click_count", { count: 0 })
  );
  await page.locator(INC_BUTTON_XPATH[xpaths]).click();
  await expect(page.locator(COUNTER_XPATH[xpaths])).toHaveText(
    maybe_td(locale, "click_count", { count: 1 })
  );
  await page.locator(INC_BUTTON_XPATH[xpaths]).click();
  await expect(page.locator(COUNTER_XPATH[xpaths])).toHaveText(
    maybe_td(locale, "click_count", { count: 2 })
  );
  await page.locator(INC_BUTTON_XPATH[xpaths]).click();
  await expect(page.locator(COUNTER_XPATH[xpaths])).toHaveText(
    maybe_td(locale, "click_count", { count: 3 })
  );
}
