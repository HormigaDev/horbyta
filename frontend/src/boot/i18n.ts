import { defineBoot } from '#q-app/wrappers';
import { createI18n } from 'vue-i18n';
import { watch } from 'vue';

import { messages as esMessages } from 'src/i18n/es';
import { messages as enMessages } from 'src/i18n/en';
import { messages as ptMessages } from 'src/i18n/pt';
import {
  applyQuasarLang,
  isSupportedLocale,
  loadSavedLocale,
  LOCALE_FALLBACK,
} from 'src/utils/locale';

export let setAppLocale: (lang: string) => void = (lang: string) => {
  applyQuasarLang(lang);
};

export default defineBoot(({ app }) => {
  const savedLocale = loadSavedLocale();

  const i18n = createI18n({
    legacy: false,
    globalInjection: true,
    locale: savedLocale,
    fallbackLocale: 'es',
    messages: {
      es: esMessages,
      en: enMessages,
      pt: ptMessages,
    },
  });

  app.use(i18n);

  setAppLocale = (lang: string) => {
    const normalizedLang = isSupportedLocale(lang) ? lang : LOCALE_FALLBACK;
    i18n.global.locale.value = normalizedLang;
    applyQuasarLang(normalizedLang);
  };

  watch(i18n.global.locale, (lang) => {
    const normalized = isSupportedLocale(lang) ? lang : LOCALE_FALLBACK;
    document.documentElement.lang = normalized === 'pt' ? 'pt-BR' : normalized;
    applyQuasarLang(normalized);
  }, { immediate: true, flush: 'sync' });
});
