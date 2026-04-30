import { Quasar } from 'quasar';
import langEs from 'quasar/lang/es';
import langEnUs from 'quasar/lang/en-US';
import langPtBr from 'quasar/lang/pt-BR';

export const LOCALE_FALLBACK = 'es';

const QUASAR_LANG_MAP = {
  es: langEs,
  en: langEnUs,
  pt: langPtBr,
} as const;

export type SupportedLocale = keyof typeof QUASAR_LANG_MAP;

export function isSupportedLocale(value: string): value is SupportedLocale {
  return value in QUASAR_LANG_MAP;
}

export function loadSavedLocale(): string {
  return LOCALE_FALLBACK;
}

export function applyQuasarLang(lang: string): void {
  const pack = isSupportedLocale(lang) ? QUASAR_LANG_MAP[lang] : QUASAR_LANG_MAP[LOCALE_FALLBACK];
  Quasar.lang.set(pack);
}
