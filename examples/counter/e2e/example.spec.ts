import { test, expect, Page } from "@playwright/test";
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

interface Input {
  page: Page;
}

async function check_english({ page }: Input) {
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

  await expect(
    page.getByText(i18nEn.click_count.replace("{{ count }}", "1"))
  ).toBeVisible();

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
}

async function check_french({ page }: Input) {
  // await page.goto("/");
  // const lngButton = page.getByRole("button", {
  //   name: i18nFr.click_to_change_lang,
  // });
  // await expect(lngButton).toBeVisible();
  // const incButton = page.getByRole("button", {
  //   name: i18nFr.click_to_inc,
  // });
  // await expect(incButton).toBeVisible();
  // await expect(
  //   page.getByText(i18nFr.click_count.replace("{{ count }}", "0"))
  // ).toBeVisible();
  // await incButton.click();
  // await expect(
  //   page.getByText(i18nFr.click_count.replace("{{ count }}", "1"))
  // ).toBeVisible();
  // await lngButton.click();
  // await expect(
  //   page.getByRole("button", {
  //     name: i18nEn.click_to_change_lang,
  //   })
  // ).toBeVisible();
  // await expect(
  //   page.getByRole("button", {
  //     name: i18nEn.click_to_inc,
  //   })
  // ).toBeVisible();
  // await expect(
  //   page.getByText(i18nEn.click_count.replace("{{ count }}", "1"))
  // ).toBeVisible();
}
