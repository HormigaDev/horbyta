import { ref } from 'vue';
import { executeRules } from 'src/utils/executeRules';
import { EMAIL_REGEX } from 'src/data/constants';
import type { Rule } from 'src/types';
import { useI18n } from 'vue-i18n';

const EMAIL_RULES: Rule<string>[] = [
  (prop: string, t) =>
    (prop.length > 255 || !EMAIL_REGEX.test(prop)) && t('rules.client.email.invalid_email'),
];

const PHONE_RULES: Rule<string>[] = [
  (prop: string, t) => !/^[0-9]+$/.test(prop) && t('rules.client.phone.invalid_phone_number'),
];

export const useClientRules = () => {
  const { t } = useI18n();
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
    return executeRules(emailRuleState, EMAIL_RULES, email, t);
  }

  function validatePhone(phone?: string) {
    if (!phone) return false;
    return executeRules(phoneRuleState, PHONE_RULES, phone, t);
  }

  return {
    emailRuleState,
    phoneRuleState,

    validateEmail,
    validatePhone,
  };
};
