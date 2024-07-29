import {
  test,
  expect,
  Page,
  Fixtures,
  TestType,
  TestFixture,
  PlaywrightWorkerArgs,
  PlaywrightWorkerOptions,
  PlaywrightTestOptions,
  PlaywrightTestArgs,
} from "@playwright/test";
import * as os from "os";

type TestParams = Parameters<Parameters<typeof test>[2]>;
export type TestArgs = TestParams[0];
export type BrowserName = TestArgs["browserName"];

const WIN = os.platform() == "win32";

export function fail_windows_webkit({ browserName }: TestArgs): boolean {
  const WEBKIT = browserName === "webkit";
  return WEBKIT && WIN;
}

class I18n {}

type TFunc = any;

export type I18nFixture = {
  i18n: I18n;
  t: TFunc;
};

function createI18nFixture(): Fixtures<
  I18nFixture,
  PlaywrightWorkerArgs & PlaywrightWorkerOptions,
  PlaywrightTestArgs & PlaywrightTestOptions,
  PlaywrightWorkerArgs & PlaywrightWorkerOptions
> {
  return {
    i18n: async ({}, use) => {
      const i18n = new I18n();
      await use(i18n);
    },
    t: async ({ i18n }, use) => {
      await use({});
    },
  };
}
