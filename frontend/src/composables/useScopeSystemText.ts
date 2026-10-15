import { useI18n } from 'vue-i18n';
import { messages } from 'src/i18n/es';

// Traduce únicamente plantillas del sistema conocidas; conserva los textos pactados en la BD.
const templateKeys = ['scope.fixedTerms', 'scope.acceptanceStatement'] as const;
export function useScopeSystemText() {
  const { t } = useI18n();
  return (value: string | null | undefined): string => {
    const key = templateKeys.find((key) => messages[key] === value);
    return key ? t(key) : (value ?? '');
  };
}
