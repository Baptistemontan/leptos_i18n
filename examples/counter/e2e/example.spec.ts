import { test, expect } from "@playwright/test";
import i18nEn from "../locales/en.json";
import i18nFr from "../locales/fr.json";

test.describe("when locale is the default locale (en-GB)", () => {
  test("simple test", check_english);
});

test.describe("when locale is set to french (fr-FR)", () => {
  test.use({
    locale: "fr",
  });

  test("simple test", check_french);
});

type TestArgs = Parameters<Parameters<typeof test>[2]>[0]; // wonky

async function check_cookie(
  context: TestArgs["context"],
  name: string,
  value: string
) {
  const cookies = await context.cookies();
  await expect(cookies.find((c) => c.name == name)?.value).toBe(value);
}

async function check_english({ page, context }: TestArgs) {
  await page.goto("/");

  await check_cookie(context, "COOKIE_PREFERED_LANG", "en");

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

  await check_cookie(context, "COOKIE_PREFERED_LANG", "fr");
}

async function check_french({ page, context }: TestArgs) {
  await page.goto("/");

  await check_cookie(context, "COOKIE_PREFERED_LANG", "en");

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

  await check_cookie(context, "COOKIE_PREFERED_LANG", "fr");
}
