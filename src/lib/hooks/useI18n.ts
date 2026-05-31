import { useCallback, useEffect, useState } from 'react';
import { t as translate, setLocale, getLocale, onLocaleChange, initI18n, type Locale } from '../i18n';

export function useI18n() {
  const [locale, setLocaleState] = useState<Locale>(() => initI18n());

  useEffect(() => {
    return onLocaleChange(() => {
      setLocaleState(getLocale());
    });
  }, []);

  const t = useCallback((key: string) => translate(key), [locale]);
  const changeLocale = useCallback((newLocale: Locale) => {
    setLocale(newLocale);
  }, []);

  return { t, locale, setLocale: changeLocale };
}
