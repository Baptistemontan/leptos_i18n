import { test, expect, Page } from "@playwright/test";
import * as os from "os";

export type TestArgs = Parameters<Parameters<typeof test>[2]>[0]; // wonky
export type BrowserName = TestArgs["browserName"];

const WIN = os.platform() == "win32";

export function fail_windows_webkit({ browserName }: TestArgs): boolean {
  const WEBKIT = browserName === "webkit";
  return WEBKIT && WIN;
}
