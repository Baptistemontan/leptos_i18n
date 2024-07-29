import {
  test,
  Fixtures,
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

type LocaleArg = {
  [key: string]: string;
};

type Locales = {
  [key: string]: LocaleArg;
};

interface I18nFixtureArgs<L extends Locales> {
  default_locale: keyof L;
  locales: L;
}

type TFuncVars = {
  [key: string]: any;
};

type TFuncComp = {
  [key: string]: (inner: string) => string;
};

type TFunc = (key: string, vars?: TFuncVars, comps?: TFuncComp) => string;
type LangChangeCb<L extends Locales> = (new_locale: keyof L) => Promise<void>;

function match_locale<L extends Locales>(
  locale: string,
  locales: L,
  default_locale: keyof L
): keyof L {
  const parts = locale.split("-");
  const keys = Object.keys(locales);

  while (parts.length) {
    const key = parts.join("-");
    if (keys.includes(key)) {
      return key as keyof L;
    }
    parts.pop();
  }

  return default_locale;
}

class I18n<L extends Locales> {
  private current_locale: keyof L;
  private locales: L;
  private locale_change_cb: LangChangeCb<L> | null;

  constructor(args: I18nFixtureArgs<L>, locale: string) {
    const { locales, default_locale } = args;

    this.current_locale = match_locale(locale, locales, default_locale);
    this.locales = locales;
  }

  public t(
    key: string,
    vars: TFuncVars = {},
    components: TFuncComp = {}
  ): string {
    let value = this.locales[this.current_locale][key];
    if (!value) {
      return key;
    }

    for (const [key, val] of Object.entries(vars)) {
      const regex = new RegExp(`{{\\s*${key}\\s*}}`, "gmi");
      value = value.replace(regex, val);
    }

    const pairs = getPairs(value);

    return formatElements(components, pairs).join("");
  }

  public get_locale(): keyof L {
    return this.current_locale;
  }

  get locale(): keyof L {
    return this.current_locale;
  }

  public async set_locale(new_locale: keyof L) {
    this.current_locale = new_locale;
    if (this.locale_change_cb) {
      await this.locale_change_cb(new_locale);
    }
  }

  public on_locale_change(cb: LangChangeCb<L>) {
    this.locale_change_cb = cb;
  }
}

export type I18nFixture<L extends Locales> = {
  i18n: I18n<L>;
  t: TFunc;
};

export function createI18nFixture<L extends Locales>(
  args: I18nFixtureArgs<L>
): Fixtures<
  I18nFixture<L>,
  PlaywrightWorkerArgs & PlaywrightWorkerOptions,
  PlaywrightTestArgs & PlaywrightTestOptions,
  PlaywrightWorkerArgs & PlaywrightWorkerOptions
> {
  return {
    i18n: async ({ locale }, use) => {
      const i18n = new I18n(args, locale);
      await use(i18n);
    },
    t: async ({ i18n }, use) => {
      await use((...args) => i18n.t(...args));
    },
  };
}

type RegexResult = [string, string, RegexResult[]]; // before, tag, content
const openTagRegex = /(<\s*?[^/>]+\s*?>)/g;
const closeTagRegex = (str: string) => new RegExp(`(</\\s*?${str}\\s*?>)`, "g");
const matchingOpenTagRegex = (str: string) =>
  new RegExp(`<\\s*?${str}\\s*?>`, "g");

function getPairs(str: string): RegexResult[] {
  const [beforeSplit, tagSplit, ...afterSplitArray] = str.split(openTagRegex);
  if (!tagSplit) {
    return [[str, "", []]];
  }
  let after = afterSplitArray?.join("") || "";
  let depth = 1;
  let content = "";
  const tag = tagSplit.slice(1, -1).trim();
  while (depth > 0) {
    const [before, _tag, ..._after] = after.split(closeTagRegex(tag));
    if (!_tag) {
      // error
      return [
        [beforeSplit + tagSplit, "", getPairs(afterSplitArray?.join("") || "")],
      ];
    }
    depth -= 1;
    depth += before.match(matchingOpenTagRegex(tag))?.length || 0;
    after = _after?.join("") || "";
    content += before;
    if (depth) {
      content += _tag;
    }
  }

  return [[beforeSplit, tag, getPairs(content)], ...getPairs(after)];
}

function join_results(results: RegexResult[]): string {
  return results.reduce((acc, result) => {
    const [a, b, rest] = result;
    return acc + a + b + join_results(rest);
  }, "");
}

function formatElements(elements: TFuncComp, parts: RegexResult[]): string[] {
  return parts.flatMap(([before, key, content]) => {
    const element = (key && elements[key]) || ((s) => s);
    return [before, element(join_results(content))];
  });
}
