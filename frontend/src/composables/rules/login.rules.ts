import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import type { Rule } from 'src/types';

const emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;

const EMAIL_RULES: Rule<string>[] = [
  (prop: string) => (!prop || prop.length < 1) && 'El correo electrónico es obligatorio.',
  (prop: string) => (prop.length > 255 || !emailRegex.test(prop)) && 'Correo electrónico inválido.',
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
