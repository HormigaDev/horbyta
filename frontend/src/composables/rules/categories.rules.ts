import type { Rule } from 'src/types';
import { executeRules } from 'src/utils/executeRules';
import { ref } from 'vue';

const NAME_RULES: Rule<string>[] = [
  (val: string) => (!val || val.length < 1) && 'El nombre es obligatorio.',
  (val: string) => val.length > 100 && 'El nombre debe tener entre 1 y 100 caracteres.',
];

export const useCategoryRules = () => {
  const nameRuleState = ref({
    error: false,
    message: '',
  });

  function validateName(name: string) {
    return executeRules(nameRuleState, NAME_RULES, name);
  }

  return {
    nameRuleState,
    validateName,
  };
};
