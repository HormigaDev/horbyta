<template>
  <q-layout view="lHh Lpr lFf">
    <q-page-container>
      <q-page class="flex flex-center q-pa-md">
        <q-card
          class="q-pa-md shadow-2"
          style="
            width: 100%;
            max-width: 480px;
            background-color: var(--card-bg);
            border: 1px solid var(--border-color);
          "
        >
          <q-card-section class="flex items-center justify-center">
            <q-img width="100px" height="100px" src="/logo.png" alt="Horbyta Logo" />
          </q-card-section>

          <q-card-section class="text-center">
            <div class="text-h5 text-weight-bold" style="color: var(--text-color)">
              {{ t('client.auth.login.brand') }}
            </div>
            <div class="text-subtitle2" style="color: var(--text-muted)">
              {{ t('client.auth.login.subtitle') }}
            </div>
          </q-card-section>

          <q-separator />

          <q-card-section class="q-pt-md">
            <q-tabs v-model="loginMode" dense align="justify" class="text-primary q-mb-sm">
              <q-tab name="credentials" :label="t('client.auth.login.tabCredentials')" />
              <q-tab name="magic_token" :label="t('client.auth.login.tabMagicToken')" />
            </q-tabs>

            <q-tab-panels v-model="loginMode" animated style="background: transparent">
              <q-tab-panel name="credentials" class="q-pa-none q-pt-sm">
                <q-form @submit.prevent="handleCredentialsLogin" class="q-gutter-md">
                  <q-input
                    v-model="email"
                    type="email"
                    outlined
                    dense
                    no-error-icon
                    autocomplete="off"
                    :label="t('client.auth.login.emailLabel')"
                    :error="loginRules.emailRuleState.value.error"
                    :error-message="loginRules.emailRuleState.value.message"
                  />

                  <q-input
                    v-model="password"
                    type="password"
                    outlined
                    dense
                    no-error-icon
                    autocomplete="off"
                    :label="t('client.auth.login.passwordLabel')"
                    :error="loginRules.passwordRuleState.value.error"
                    :error-message="loginRules.passwordRuleState.value.message"
                  />

                  <q-btn
                    type="submit"
                    color="primary"
                    class="full-width"
                    :label="t('client.auth.login.submitCredentials')"
                    :loading="isLoadingCredentials"
                  />
                </q-form>
              </q-tab-panel>

              <q-tab-panel name="magic_token" class="q-pa-none q-pt-sm">
                <q-form @submit.prevent="handleMagicTokenLogin" class="q-gutter-md">
                  <q-input
                    v-model="magicToken"
                    type="textarea"
                    autogrow
                    outlined
                    dense
                    no-error-icon
                    :label="t('client.auth.login.magicTokenLabel')"
                  />

                  <div class="text-caption" style="color: var(--text-muted)">
                    {{ t('client.auth.login.magicTokenHint') }}
                  </div>

                  <q-btn
                    type="submit"
                    color="secondary"
                    class="full-width"
                    :label="t('client.auth.login.submitMagicToken')"
                    :loading="isLoadingMagicToken"
                  />
                </q-form>
              </q-tab-panel>
            </q-tab-panels>

            <div v-if="errorMessage" class="text-negative text-caption text-center q-mt-md">
              {{ errorMessage }}
            </div>
          </q-card-section>
        </q-card>
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useLoginRules } from 'src/composables/rules/login.rules';
import { useClientAuthStore } from 'src/stores/clientAuth';

type LoginMode = 'credentials' | 'magic_token';

const router = useRouter();
const route = useRoute();
const { t } = useI18n();
const clientAuthStore = useClientAuthStore();
const loginRules = useLoginRules();

const loginMode = ref<LoginMode>('credentials');
const email = ref('');
const password = ref('');
const magicToken = ref('');
const isLoadingCredentials = ref(false);
const isLoadingMagicToken = ref(false);
const errorMessage = ref('');
const portalUserId = computed(() => {
  const raw = route.params.userId;
  return typeof raw === 'string' ? raw.trim() : '';
});

const parseErrorMessage = (err: unknown): string => {
  if (err instanceof Error && err.message) {
    return err.message;
  }

  return t('client.auth.login.unexpectedError');
};

const handleCredentialsLogin = async () => {
  errorMessage.value = '';
  isLoadingCredentials.value = true;

  if (loginRules.validateEmail(email.value) || loginRules.validatePassword(password.value)) {
    isLoadingCredentials.value = false;
    password.value = '';
    return;
  }

  try {
    await clientAuthStore.login({
      user_id: portalUserId.value,
      email: email.value.trim(),
      password: password.value,
    });
    void router.push('/client');
  } catch (err: unknown) {
    errorMessage.value = parseErrorMessage(err);
  } finally {
    isLoadingCredentials.value = false;
    password.value = '';
  }
};

const handleMagicTokenLogin = async () => {
  errorMessage.value = '';
  isLoadingMagicToken.value = true;

  if (!magicToken.value.trim()) {
    errorMessage.value = t('client.auth.login.tokenRequired');
    isLoadingMagicToken.value = false;
    return;
  }

  try {
    await clientAuthStore.loginWithMagicToken({
      user_id: portalUserId.value,
      magic_token: magicToken.value.trim(),
    });
    void router.push('/client');
  } catch (err: unknown) {
    errorMessage.value = parseErrorMessage(err);
  } finally {
    isLoadingMagicToken.value = false;
    magicToken.value = '';
  }
};
</script>
