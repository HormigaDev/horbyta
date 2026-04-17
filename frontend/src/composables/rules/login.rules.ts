import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import { EMAIL_REGEX } from 'src/data/constants';
import type { Rule } from 'src/types';
import { useI18n } from 'vue-i18n';

const EMAIL_RULES: Rule<string>[] = [
  (prop: string, t) => (!prop || prop.length < 1) && t('rules.login.email.required'),
  (prop: string, t) =>
    (prop.length > 255 || !EMAIL_REGEX.test(prop)) && t('rules.login.email.invalid_email'),
];

const PASSWORD_RULES: Rule<string>[] = [
  (password: string, t) => !password && t('rules.login.password.required'),
];

export const useLoginRules = () => {
  const { t } = useI18n();
  const emailRuleState = ref({
    error: false,
    message: '',
  });
  const passwordRuleState = ref({
    error: false,
    message: '',
  });

  function validateEmail(email: string) {
    return executeRules(emailRuleState, EMAIL_RULES, email, t);
  }
  function validatePassword(password: string) {
    return executeRules(passwordRuleState, PASSWORD_RULES, password, t);
  }

  return {
    emailRuleState,
    passwordRuleState,

    validateEmail,
    validatePassword,
  };
};
