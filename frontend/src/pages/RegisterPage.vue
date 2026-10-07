<template>
  <AuthFrame :title="t('auth.register.submit')" :subtitle="t('auth.register.subtitle')">
    <q-card-section>
      <q-form @submit.prevent="handleRegister" class="q-gutter-md">
        <q-input
          v-model="name"
          name="name"
          autocomplete="name"
          autofocus
          type="text"
          :label="t('auth.register.nameLabel')"
          outlined
          dense
          no-error-icon
          :error="registerRules.nameRuleState.value.error"
          :error-message="registerRules.nameRuleState.value.message"
        >
          <template #append>
            <q-icon name="emergency" style="font-size: 12px" />
          </template>
        </q-input>

        <q-input
          v-model="email"
          type="email"
          name="email"
          autocomplete="username"
          autocapitalize="none"
          spellcheck="false"
          :label="t('auth.register.emailLabel')"
          outlined
          dense
          no-error-icon
          :error="registerRules.emailRuleState.value.error"
          :error-message="registerRules.emailRuleState.value.message"
        >
          <template #append>
            <q-icon name="emergency" style="font-size: 12px" />
          </template>
        </q-input>

        <q-input
          v-model="password"
          type="password"
          name="password"
          autocomplete="new-password"
          :label="t('auth.register.passwordLabel')"
          outlined
          dense
          no-error-icon
          :error="registerRules.passwordRuleState.value.error"
          :error-message="registerRules.passwordRuleState.value.message"
        >
          <template #append>
            <q-icon name="emergency" style="font-size: 12px" />
          </template>
        </q-input>

        <q-input
          v-model="confirmPassword"
          type="password"
          name="password-confirmation"
          autocomplete="new-password"
          :label="t('auth.register.confirmPasswordLabel')"
          outlined
          dense
          no-error-icon
          :error="registerRules.confirmPasswordRuleState.value.error"
          :error-message="registerRules.confirmPasswordRuleState.value.message"
        >
          <template #append>
            <q-icon name="emergency" style="font-size: 12px" />
          </template>
        </q-input>

        <div v-if="errorMessage" role="alert" class="text-negative text-center text-caption">
          {{ errorMessage }}
        </div>

        <div>
          <q-btn
            :label="t('auth.register.submit')"
            type="submit"
            unelevated
            color="primary"
            class="full-width"
            :loading="isLoading"
          />
        </div>
      </q-form>
    </q-card-section>

    <q-card-section class="text-center q-pt-none">
      <span class="text-caption text-grey-7">{{ t('auth.register.alreadyHaveAccount') }} </span>
      <router-link to="/login" class="q-ml-sm text-caption text-primary">
        {{ t('auth.register.loginLink') }}
      </router-link>
    </q-card-section>
  </AuthFrame>
</template>
<script setup lang="ts">
import AuthFrame from 'src/components/AuthFrame.vue';
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from 'src/stores/auth';
import { apiFetch, isApiHttpError } from 'src/boot/fetch';
import { useRegisterRules } from 'src/composables/rules/register.rules';
import { RouterLink } from 'vue-router';

const { t } = useI18n();
const router = useRouter();
const authStore = useAuthStore();

const registerRules = useRegisterRules();

const name = ref('');
const email = ref('');
const password = ref('');
const confirmPassword = ref('');
const isLoading = ref(false);
const errorMessage = ref('');

async function handleRegister() {
  isLoading.value = true;
  errorMessage.value = '';

  const hasErrors =
    registerRules.validateName(name.value) ||
    registerRules.validateEmail(email.value) ||
    registerRules.validatePassword(password.value) ||
    registerRules.validateConfirmPassword(password.value, confirmPassword.value);

  if (hasErrors) {
    isLoading.value = false;
    password.value = '';
    confirmPassword.value = '';
    return;
  }

  try {
    const response = await apiFetch('/auth/register', {
      method: 'POST',
      body: JSON.stringify({
        name: name.value,
        email: email.value,
        password: password.value,
      }),
    });

    authStore.setAuthenticated(true, response.user);
    await router.push({ name: 'dashboard' });
  } catch (error) {
    if (isApiHttpError(error)) {
      if (error.status === 409) {
        errorMessage.value = t('auth.register.conflictError');
      } else if (error.status === 423) {
        errorMessage.value = t('auth.register.lockedError');
      } else {
        errorMessage.value = t('auth.register.unexpectedError');
      }
    } else {
      errorMessage.value = t('auth.register.unexpectedError');
    }
  } finally {
    isLoading.value = false;
    password.value = '';
    confirmPassword.value = '';
  }
}
</script>
