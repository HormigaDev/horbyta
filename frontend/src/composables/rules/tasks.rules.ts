import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import type { Rule } from 'src/types';
import { useI18n } from 'vue-i18n';

const TITLE_RULES: Rule<string>[] = [
  (val: string, t) => (!val || val.length < 1) && t('rules.task.title.required'),
  (val: string, t) => val.length > 255 && t('rules.task.title.limit'),
];

export const useTasksRules = () => {
  const { t } = useI18n();
  const titleRuleState = ref({
    error: false,
    message: '',
  });

  const validateTitle = (title: string) => executeRules(titleRuleState, TITLE_RULES, title, t);

  return {
    titleRuleState,
    validateTitle,
  };
};
