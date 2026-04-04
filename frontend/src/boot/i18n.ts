import { defineBoot } from '#q-app/wrappers';
import { createI18n } from 'vue-i18n';

import { messages as esMessages } from 'src/i18n/es';

export default defineBoot(({ app }) => {
  const i18n = createI18n({
    legacy: false,
    globalInjection: true,
    locale: 'es',
    fallbackLocale: 'es',
    messages: {
      es: esMessages,
    },
  });

  app.use(i18n);
});
