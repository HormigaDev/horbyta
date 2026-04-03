import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import type { Rule } from 'src/types';

const TITLE_RULES: Rule<string>[] = [
  (val: string) => (!val || val.length < 1) && 'El título es obligatorio.',
  (val: string) => val.length > 255 && 'El título debe tener entre 1 y 255 caracteres.',
];

export const useTasksRules = () => {
  const titleRuleState = ref({
    error: false,
    message: '',
  });

  const validateTitle = (title: string) => executeRules(titleRuleState, TITLE_RULES, title);

  return {
    titleRuleState,
    validateTitle,
  };
};
