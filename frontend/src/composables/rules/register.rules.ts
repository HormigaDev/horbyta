import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import { EMAIL_REGEX, CLIENT_PASSWORD_REGEX } from 'src/data/constants';
import type { Rule } from 'src/types';
import { useI18n } from 'vue-i18n';

const NAME_RULES: Rule<string>[] = [
  (prop: string, t) => (!prop || prop.length < 1) && t('rules.register.name.required'),
  (prop: string, t) => prop.length < 2 && t('rules.register.name.min'),
  (prop: string, t) => prop.length > 100 && t('rules.register.name.max'),
];

const EMAIL_RULES: Rule<string>[] = [
  (prop: string, t) => (!prop || prop.length < 1) && t('rules.register.email.required'),
  (prop: string, t) =>
    (prop.length > 255 || !EMAIL_REGEX.test(prop)) && t('rules.register.email.invalid_email'),
];

const PASSWORD_RULES: Rule<string>[] = [
  (prop: string, t) => (!prop || prop.length < 1) && t('rules.register.password.required'),
  (prop: string, t) => !CLIENT_PASSWORD_REGEX.test(prop) && t('rules.register.password.weak'),
];

export const useRegisterRules = () => {
  const { t } = useI18n();

  const nameRuleState = ref({ error: false, message: '' });
  const emailRuleState = ref({ error: false, message: '' });
  const passwordRuleState = ref({ error: false, message: '' });

  function validateName(name: string) {
    return executeRules(nameRuleState, NAME_RULES, name, t);
  }

  function validateEmail(email: string) {
    return executeRules(emailRuleState, EMAIL_RULES, email, t);
  }

  function validatePassword(password: string) {
    return executeRules(passwordRuleState, PASSWORD_RULES, password, t);
  }

  return {
    nameRuleState,
    emailRuleState,
    passwordRuleState,

    validateName,
    validateEmail,
    validatePassword,
  };
};
