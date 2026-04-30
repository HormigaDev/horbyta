import { defineBoot } from '#q-app/wrappers';
import { createI18n } from 'vue-i18n';

import { messages as esMessages } from 'src/i18n/es';
import { messages as enMessages } from 'src/i18n/en';
import { messages as ptMessages } from 'src/i18n/pt';
import { applyQuasarLang, loadSavedLocale } from 'src/utils/locale';

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

  applyQuasarLang(savedLocale);
});
