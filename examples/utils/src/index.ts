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

type PluralType =
  | "i8"
  | "i16"
  | "i32"
  | "i64"
  | "u8"
  | "u16"
  | "u32"
  | "u64"
  | "f32"
  | "f64";
type Plural = [string, ...(number | string)[]];
type RealPlurals = [PluralType | Plural, ...Plural[]];
type InferedPlurals = (number | string)[][];

type LocaleValue =
  | {
      [key: string]: LocaleValue;
    }
  | string
  | InferedPlurals;

type Locales = {
  [key: string]: LocaleValue;
};

interface I18nFixtureArgsNoNs<L extends Locales> {
  default_locale: keyof L;
  locales: L;
}

type Namespaces<L extends Locales> = {
  [key: string]: L;
};

interface I18nFixtureArgsWithNs<L extends Locales, N extends Namespaces<L>> {
  default_locale: keyof L;
  namespaces: N;
}

type I18nFixtureArgs<L extends Locales, N extends Namespaces<L>> =
  | I18nFixtureArgsNoNs<L>
  | I18nFixtureArgsWithNs<L, N>;

type TFuncVars = {
  [key: string]: any;
};

type TFuncComp = {
  [key: string]: (inner: string) => string;
};

type ShiftTuple<T extends any[]> = T extends [T[0], ...infer R] ? R : never;

export type TDFuncParams<L extends Locales> = [
  locale: keyof L,
  key: string,
  vars?: TFuncVars,
  components?: TFuncComp,
  plural_count?: number
];

export type TDFunc<L extends Locales> = (...args: TDFuncParams<L>) => string;

export type TFuncParams<L extends Locales> = ShiftTuple<TDFuncParams<L>>;

export type TFunc<L extends Locales> = (...args: TFuncParams<L>) => string;

export type MaybeTd<L extends Locales> = (
  locale: keyof L | undefined,
  ...args: TFuncParams<L>
) => string;

export type GetL<T> = T extends I18n<infer L, any> ? L : never;

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

function is_namespace_args<L extends Locales, N extends Namespaces<L>>(
  args: I18nFixtureArgs<L, N>
): args is I18nFixtureArgsWithNs<L, N> {
  return Object.keys(args).includes("namespaces");
}

class I18n<L extends Locales, N extends Namespaces<L>> {
  private current_locale: keyof L;
  private default_locale: keyof L;
  private locales: L;
  private locale_change_cb: LangChangeCb<L> | null;

  constructor(args: I18nFixtureArgs<L, N>, locale: string) {
    let updated_args: I18nFixtureArgsNoNs<L>;
    if (is_namespace_args(args)) {
      const locales = {};
      for (const nkey of Object.keys(args.namespaces)) {
        const ns = args.namespaces[nkey];
        for (const lkey of Object.keys(ns)) {
          if (!locales[lkey]) {
            locales[lkey] = {};
          }
          locales[lkey][nkey] = ns[lkey];
        }
      }
      updated_args = {
        default_locale: args.default_locale,
        locales: locales as L,
      };
    } else {
      updated_args = args;
    }

    const { locales, default_locale } = updated_args;

    this.current_locale = match_locale(locale, locales, default_locale);
    this.default_locale = default_locale;
    this.locales = locales;
  }

  public t(...args: TFuncParams<L>): string {
    return this.td(this.current_locale, ...args);
  }

  public td(
    locale: keyof L,
    key: string,
    vars: TFuncVars = {},
    components: TFuncComp = {},
    plural_count?: number
  ): string {
    if (typeof plural_count !== "undefined") {
      vars["count"] = plural_count;
    }
    let value = key
      .split(".")
      .reduce<LocaleValue | undefined>(
        (vals, key) => (vals ? vals[key] : undefined),
        this.locales[locale]
      );
    if (typeof value === "string") {
      return handle_string(value, vars, components);
    } else if (Array.isArray(value)) {
      return handle_plurals(
        value as RealPlurals,
        vars,
        components,
        plural_count
      );
    } else {
      throw new Error(`invalid key \"${key}\",\nvalue: ${value}`);
    }
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

  public set_locale_untracked(new_locale: keyof L) {
    this.current_locale = new_locale;
  }

  public on_locale_change(cb: LangChangeCb<L>) {
    this.locale_change_cb = cb;
  }

  public get_url(path?: string): string {
    let stripped_path = path || "";
    while (stripped_path.startsWith("/")) {
      stripped_path = stripped_path.substring(1);
    }
    stripped_path = stripped_path ? "/" + stripped_path : "";
    if (this.current_locale == this.default_locale) {
      return stripped_path;
    } else {
      const locale_part = "/" + (this.current_locale as string);
      return locale_part + stripped_path;
    }
  }
}

export type I18nFixture<L extends Locales, N extends Namespaces<L>> = {
  i18n: I18n<L, N>;
  t: TFunc<L>;
  maybe_td: MaybeTd<L>;
};

export function createI18nFixture<L extends Locales, N extends Namespaces<L>>(
  args: I18nFixtureArgs<L, N>
): Fixtures<
  I18nFixture<L, N>,
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
    maybe_td: async ({ i18n }, use) => {
      await use((locale, ...args) =>
        locale ? i18n.td(locale, ...args) : i18n.t(...args)
      );
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

function handle_string(
  value: string,
  vars: TFuncVars,
  components: TFuncComp
): string {
  for (const [key, val] of Object.entries(vars)) {
    const regex = new RegExp(`{{\\s*${key}\\s*}}`, "gmi");
    value = value.replace(regex, val);
  }

  const pairs = getPairs(value);

  return formatElements(components, pairs).join("");
}

function string_plural_match(plural: string, plural_count: number): boolean {
  const counts = plural.split("|").map((s) => s.trim());
  for (const count of counts) {
    if (count.includes("..")) {
      // range
      let [before, after] = count.split("..", 2);
      const included = after.startsWith("=");
      if (included) {
        after = after.slice(1);
      }
      let min = Number.parseFloat(before);
      let max = Number.parseFloat(after);

      if (isNaN(min)) {
        min = -Infinity;
      }
      if (isNaN(max)) {
        max = Infinity;
      }

      if (
        (plural_count >= min && included && plural_count <= max) ||
        (!included && plural_count < max)
      ) {
        return true;
      }
    } else {
      const parseNum = Number.parseFloat(count);
      if (parseNum == plural_count) {
        return true;
      }
    }
  }
  return false;
}

function plural_match(plural: Plural, plural_count?: number): boolean {
  let [, ...counts] = plural;
  if (!counts.length || typeof plural_count !== "number") {
    return !counts.length;
  }

  for (const count of counts) {
    if (
      count == plural_count ||
      (typeof count === "string" && string_plural_match(count, plural_count))
    ) {
      return true;
    }
  }

  return false;
}

function handle_plurals(
  value: RealPlurals,
  vars: TFuncVars,
  components: TFuncComp,
  plural_count?: number
): string {
  const [first, ...rest] = value;

  if (typeof first !== "string") {
    rest.unshift(first);
  }

  for (const plural of rest) {
    if (plural_match(plural, plural_count)) {
      return handle_string(plural[0], vars, components);
    }
  }

  throw new Error(
    `plurals should match.\ncount: ${plural_count}\nplurals: ${JSON.stringify(
      value
    )}`
  );
}
