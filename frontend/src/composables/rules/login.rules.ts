import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import type { Rule } from 'src/types';
import { EMAIL_REGEX } from 'src/data/constants';

const EMAIL_RULES: Rule<string>[] = [
  (prop: string) => (!prop || prop.length < 1) && 'El correo electrónico es obligatorio.',
  (prop: string) =>
    (prop.length > 255 || !EMAIL_REGEX.test(prop)) && 'Correo electrónico inválido.',
];

const PASSWORD_RULES: Rule<string>[] = [
  (password: string) => !password && 'La contraseña es obligatoria.',
];

export const useLoginRules = () => {
  const emailRuleState = ref({
    error: false,
    message: '',
  });
  const passwordRuleState = ref({
    error: false,
    message: '',
  });

  function validateEmail(email: string) {
    return executeRules(emailRuleState, EMAIL_RULES, email);
  }
  function validatePassword(password: string) {
    return executeRules(passwordRuleState, PASSWORD_RULES, password);
  }

  return {
    emailRuleState,
    passwordRuleState,

    validateEmail,
    validatePassword,
  };
};
