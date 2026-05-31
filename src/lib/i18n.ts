export type Locale = 'en' | 'zh-CN';

interface TranslationDict {
  [key: string]: string | TranslationDict;
}

function getNestedValue(obj: TranslationDict, path: string): string {
  const keys = path.split('.');
  let current: string | TranslationDict = obj;
  for (const key of keys) {
    if (typeof current !== 'object' || current === null) return path;
    current = current[key];
    if (current === undefined) return path;
  }
  return typeof current === 'string' ? current : path;
}

let currentLocale: Locale = 'en';
let translations: Partial<Record<Locale, TranslationDict>> = {};
let onChangeCallbacks: Array<() => void> = [];

export function setLocale(locale: Locale) {
  currentLocale = locale;
  localStorage.setItem('forge-env-locale', locale);
  onChangeCallbacks.forEach((cb) => cb());
}

export function getLocale(): Locale {
  return currentLocale;
}

export function onLocaleChange(callback: () => void): () => void {
  onChangeCallbacks.push(callback);
  return () => {
    onChangeCallbacks = onChangeCallbacks.filter((cb) => cb !== callback);
  };
}

export function t(key: string): string {
  const dict = translations[currentLocale];
  if (!dict) return key;
  return getNestedValue(dict, key);
}

export function loadTranslations(locale: Locale, dict: TranslationDict) {
  translations = { ...translations, [locale]: dict };
}

export function initI18n(): Locale {
  const stored = localStorage.getItem('forge-env-locale') as Locale | null;
  if (stored && (stored === 'en' || stored === 'zh-CN')) {
    currentLocale = stored;
    return stored;
  }
  // Detect from browser
  const browserLang = navigator.language;
  if (browserLang.startsWith('zh')) {
    currentLocale = 'zh-CN';
  } else {
    currentLocale = 'en';
  }
  return currentLocale;
}
