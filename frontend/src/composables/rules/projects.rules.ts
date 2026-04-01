import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import type { Rule } from 'src/types';

const NAME_RULES: Rule<string>[] = [
  (val: string) => (!val || val.length < 1) && 'El nombre es obligatorio.',
  (val: string) => val.length > 255 && 'El nombre debe tener entre 1 y 255 caracteres.',
];

const DESCRIPTION_RULES: Rule<string | null>[] = [
  (val: string | null) =>
    val !== null && val.length > 15000 && 'La descripción no puede exceder los 15000 caracteres.',
];

export const useProjectsRules = () => {
  const nameRuleState = ref({
    error: false,
    message: '',
  });

  const descriptionRuleState = ref({
    error: false,
    message: '',
  });

  function validateName(name: string) {
    return executeRules(nameRuleState, NAME_RULES, name);
  }

  function validateDescription(description: string | null) {
    return executeRules(descriptionRuleState, DESCRIPTION_RULES, description);
  }

  return {
    nameRuleState,
    descriptionRuleState,

    validateName,
    validateDescription,
  };
};
