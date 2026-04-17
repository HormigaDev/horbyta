import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import type { Rule } from 'src/types';
import { useI18n } from 'vue-i18n';

const NAME_RULES: Rule<string>[] = [
  (val: string, t) => (!val || val.length < 1) && t('rules.project.name.required'),
  (val: string, t) => val.length > 255 && t('rules.project.name.limit'),
];

const DESCRIPTION_RULES: Rule<string | null>[] = [
  (val: string | null, t) =>
    val !== null && val.length > 15000 && t('rules.project.description.limit'),
];

export const useProjectsRules = () => {
  const { t } = useI18n();
  const nameRuleState = ref({
    error: false,
    message: '',
  });

  const descriptionRuleState = ref({
    error: false,
    message: '',
  });

  function validateName(name: string) {
    return executeRules(nameRuleState, NAME_RULES, name, t);
  }

  function validateDescription(description: string | null) {
    return executeRules(descriptionRuleState, DESCRIPTION_RULES, description, t);
  }

  return {
    nameRuleState,
    descriptionRuleState,

    validateName,
    validateDescription,
  };
};
