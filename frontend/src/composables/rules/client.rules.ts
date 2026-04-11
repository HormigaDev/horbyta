import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import type { Rule } from 'src/types';
import { EMAIL_REGEX } from 'src/data/constants';

const EMAIL_RULES: Rule<string>[] = [
  (prop: string) =>
    (prop.length > 255 || !EMAIL_REGEX.test(prop)) && 'Correo electrónico inválido.',
];

const PHONE_RULES: Rule<string>[] = [
  (prop: string) => !/^[0-9]+$/.test(prop) && 'Número de teléfono inválido',
];

export const useClientRules = () => {
  const emailRuleState = ref({
    error: false,
    message: '',
  });
  const phoneRuleState = ref({
    error: false,
    message: '',
  });

  function validateEmail(email?: string) {
    if (!email) return false;
    return executeRules(emailRuleState, EMAIL_RULES, email);
  }

  function validatePhone(phone?: string) {
    if (!phone) return false;
    return executeRules(phoneRuleState, PHONE_RULES, phone);
  }

  return {
    emailRuleState,
    phoneRuleState,

    validateEmail,
    validatePhone,
  };
};
