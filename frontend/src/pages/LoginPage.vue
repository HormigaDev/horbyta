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
            <div class="text-subtitle2 text-grey-7">
              {{ isTwoFactorStep ? t('auth.login.twoFactorSubtitle') : t('auth.login.subtitle') }}
            </div>
          </q-card-section>

          <q-card-section>
            <q-form @submit.prevent="handleLogin" class="q-gutter-md">
              <q-input
                v-if="!isTwoFactorStep"
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
                v-if="!isTwoFactorStep"
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

              <template v-if="isTwoFactorStep">
                <q-tabs v-model="twoFactorMode" dense align="justify" narrow-indicator>
                  <q-tab name="totp" icon="password" :label="t('auth.login.twoFactorTab')" />
                  <q-tab name="backup" icon="vpn_key" :label="t('auth.login.backupCodeTab')" />
                </q-tabs>

                <q-input
                  v-if="twoFactorMode === 'totp'"
                  v-model="twoFactorCode"
                  :label="t('auth.login.twoFactorCodeLabel')"
                  outlined
                  dense
                  no-error-icon
                  autocomplete="one-time-code"
                  inputmode="numeric"
                  maxlength="8"
                />

                <q-input
                  v-else
                  v-model="backupCode"
                  :label="t('auth.login.backupCodeLabel')"
                  outlined
                  dense
                  no-error-icon
                  autocomplete="one-time-code"
                />
              </template>

              <div v-if="errorMessage" class="text-negative text-center text-caption">
                {{ errorMessage }}
              </div>

              <q-btn
                v-if="isTwoFactorStep"
                flat
                color="primary"
                class="full-width"
                :label="t('auth.login.backToCredentials')"
                @click="resetTwoFactorStep"
              />

              <div>
                <q-btn
                  :label="isTwoFactorStep ? t('auth.login.verifyTwoFactor') : t('auth.login.submit')"
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
const twoFactorCode = ref('');
const backupCode = ref('');
const twoFactorMode = ref<'totp' | 'backup'>('totp');
const isTwoFactorStep = ref(false);
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

  if (!isTwoFactorStep.value && (loginRules.validateEmail(email.value) || loginRules.validatePassword(password.value))) {
    isLoading.value = false;
    password.value = '';
    return;
  }

  if (isTwoFactorStep.value && twoFactorMode.value === 'totp' && !twoFactorCode.value.trim()) {
    errorMessage.value = t('auth.login.twoFactorRequired');
    isLoading.value = false;
    return;
  }

  if (isTwoFactorStep.value && twoFactorMode.value === 'backup' && !backupCode.value.trim()) {
    errorMessage.value = t('auth.login.backupCodeRequired');
    isLoading.value = false;
    return;
  }

  try {
    const response = await apiFetch('/auth/login', {
      method: 'POST',
      body: JSON.stringify({
        email: email.value,
        password: password.value,
        two_factor_auth_code:
          isTwoFactorStep.value && twoFactorMode.value === 'totp'
            ? twoFactorCode.value.trim()
            : undefined,
        two_factor_auth_backup_code:
          isTwoFactorStep.value && twoFactorMode.value === 'backup'
            ? backupCode.value.trim()
            : undefined,
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
    if (isApiHttpError(error) && error.type === 'two_factor_auth_code_required') {
      isTwoFactorStep.value = true;
      errorMessage.value = '';
    } else if (
      isApiHttpError(error) &&
      (error.type === 'invalid_two_factor_auth_code' || error.type === 'replayed_two_factor_auth_code')
    ) {
      errorMessage.value = t('auth.login.invalidTwoFactorError');
    } else if (isApiHttpError(error) && error.status === 429) {
      errorMessage.value = t('error.rateLimited');
    } else if (isApiHttpError(error) && error.status === 401) {
      errorMessage.value = t('auth.login.invalidCredentialsError');
      password.value = '';
    } else {
      errorMessage.value = t('auth.login.unexpectedError');
    }
  } finally {
    isLoading.value = false;
    twoFactorCode.value = '';
    backupCode.value = '';
  }
}

function resetTwoFactorStep() {
  isTwoFactorStep.value = false;
  twoFactorMode.value = 'totp';
  twoFactorCode.value = '';
  backupCode.value = '';
  password.value = '';
  errorMessage.value = '';
}
</script>
