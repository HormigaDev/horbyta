<template>
  <q-layout view="lHh Lpr lFf">
    <q-page-container>
      <q-page class="flex flex-center">
        <q-card class="q-pa-md shadow-2" style="width: 100%; max-width: 400px">
          <q-card-section class="flex items-center justify-center">
            <q-img width="100px" height="100px" src="/logo.png" alt="Horbyta Logo" />
          </q-card-section>

          <q-card-section class="text-center">
            <div class="text-h5 text-weight-bold">{{ t('auth.login.brand') }}</div>
            <div class="text-subtitle2 text-grey-7">{{ t('auth.login.subtitle') }}</div>
          </q-card-section>

          <q-card-section>
            <q-form @submit.prevent="handleLogin" class="q-gutter-md">
              <q-input
                v-model="email"
                type="email"
                :label="t('auth.login.emailLabel')"
                outlined
                dense
                no-error-icon
                autocomplete="off"
                :error="loginRules.emailRuleState.value.error"
                :error-message="loginRules.emailRuleState.value.message"
              >
                <template #append>
                  <q-icon name="emergency" style="font-size: 12px" />
                </template>
              </q-input>
              <q-input
                v-model="password"
                type="password"
                :label="t('auth.login.passwordLabel')"
                outlined
                dense
                no-error-icon
                autocomplete="off"
                :error="loginRules.passwordRuleState.value.error"
                :error-message="loginRules.passwordRuleState.value.message"
              >
                <template #append>
                  <q-icon name="emergency" style="font-size: 12px" />
                </template>
              </q-input>

              <div v-if="errorMessage" class="text-negative text-center text-caption">
                {{ errorMessage }}
              </div>

              <div>
                <q-btn
                  :label="t('auth.login.submit')"
                  type="submit"
                  color="primary"
                  class="full-width"
                  :loading="isLoading"
                />
              </div>
            </q-form>
          </q-card-section>

          <q-card-section v-if="registrationOpen" class="text-center q-pt-none">
            <span class="text-caption text-grey-7">{{ t('auth.login.noAccount') }} </span>
            <router-link to="/register" class="q-ml-sm text-caption text-primary">
              {{ t('auth.login.registerLink') }}
            </router-link>
          </q-card-section>
        </q-card>
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from 'src/stores/auth';
import { apiFetch, isApiHttpError } from 'src/boot/fetch';
import { useLoginRules } from 'src/composables/rules/login.rules';
import { RouterLink } from 'vue-router';
import { UsersService } from 'src/services/users';
import { setAppLocale } from 'src/boot/i18n';

const { t, locale } = useI18n();
const router = useRouter();
const authStore = useAuthStore();

const loginRules = useLoginRules();

const email = ref('');
const password = ref('');
const isLoading = ref(false);
const errorMessage = ref('');
const registrationOpen = ref(false);

onMounted(async () => {
  try {
    const status = await apiFetch('/auth/registration-status');
    registrationOpen.value = status.open === true;
  } catch {
    registrationOpen.value = false;
  }
});

async function handleLogin() {
  isLoading.value = true;
  errorMessage.value = '';

  if (loginRules.validateEmail(email.value) || loginRules.validatePassword(password.value)) {
    isLoading.value = false;
    password.value = '';
    return;
  }

  try {
    const response = await apiFetch('/auth/login', {
      method: 'POST',
      body: JSON.stringify({
        email: email.value,
        password: password.value,
      }),
    });

    authStore.setAuthenticated(true, response.user);

    try {
      const settings = await UsersService.getMe();
      const lang = settings.preferences?.language;
      if (lang) {
        locale.value = lang;
        setAppLocale(lang);
      }
    } catch {
      // non-blocking: si falla, se mantiene el locale por defecto.
    }

    await router.push({ name: 'dashboard' });
  } catch (error) {
    if (isApiHttpError(error) && error.status === 401) {
      errorMessage.value = t('auth.login.invalidCredentialsError');
    } else {
      errorMessage.value = t('auth.login.unexpectedError');
    }
  } finally {
    isLoading.value = false;
    password.value = '';
  }
}
</script>
