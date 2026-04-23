import type { Rule } from 'src/types';
import { executeRules } from 'src/utils/executeRules';
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';

const NAME_RULES: Rule<string>[] = [
  (val: string, t) => (!val || val.length < 1) && t('rules.activity.name.required'),
  (val: string, t) => val.length > 100 && t('rules.activity.name.limit'),
];

export const useActivityRules = () => {
  const { t } = useI18n();
  const nameRuleState = ref({
    error: false,
    message: '',
  });

  function validateName(name: string) {
    return executeRules(nameRuleState, NAME_RULES, name, t);
  }

  return {
    nameRuleState,
    validateName,
  };
};
