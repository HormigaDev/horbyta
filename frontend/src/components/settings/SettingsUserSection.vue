<template>
  <q-card flat bordered>
    <q-card-section>
      <div class="text-h6">{{ t('settings.user.title') }}</div>
      <div class="text-caption" style="color: var(--text-muted)">
        {{ t('settings.user.subtitle') }}
      </div>
    </q-card-section>

    <q-separator />

    <q-card-section class="row q-col-gutter-md">
      <div class="col-12 col-md-6">
        <q-input v-model="profileForm.name" :label="t('settings.user.nameLabel')" outlined dense />
      </div>
      <div class="col-12 col-md-6">
        <q-input
          v-model="profileForm.email"
          :label="t('settings.user.emailLabel')"
          type="email"
          outlined
          dense
        />
      </div>
    </q-card-section>

    <q-card-actions align="right">
      <q-btn
        color="primary"
        unelevated
        :label="t('settings.user.saveProfileButton')"
        :loading="isSavingProfile"
        @click="saveProfile"
      />
    </q-card-actions>

    <q-separator />

    <q-card-section>
      <div class="row items-center justify-between q-col-gutter-md">
        <div class="col-12 col-md">
          <div class="text-subtitle1 q-mb-xs">{{ t('settings.user.twoFactorTitle') }}</div>
          <div class="text-caption" style="color: var(--text-muted)">
            {{ t('settings.user.twoFactorSubtitle') }}
          </div>
        </div>
        <div class="col-12 col-md-auto">
          <q-chip
            square
            :color="totpStatus?.enabled ? 'positive' : 'grey'"
            :icon="totpStatus?.enabled ? 'verified_user' : 'shield'"
            :label="
              totpStatus?.enabled
                ? t('settings.user.twoFactorEnabled')
                : t('settings.user.twoFactorDisabled')
            "
          />
        </div>
      </div>

      <div v-if="totpStatus?.enabled" class="text-caption q-mt-sm" style="color: var(--text-muted)">
        {{
          t('settings.user.backupCodesRemaining', {
            count: totpStatus.backup_codes_remaining,
          })
        }}
      </div>
    </q-card-section>

    <q-card-actions align="right">
      <q-btn
        v-if="!totpStatus?.enabled"
        color="primary"
        unelevated
        icon="add_moderator"
        :label="t('settings.user.startTwoFactorButton')"
        :loading="isLoadingTotp"
        @click="startTotpSetup"
      />
      <q-btn
        v-else
        color="negative"
        outline
        icon="shield"
        :label="t('settings.user.disableTwoFactorButton')"
        @click="isDisableTotpDialogOpen = true"
      />
    </q-card-actions>

    <q-separator />

    <q-card-section>
      <div class="text-subtitle1 q-mb-sm">{{ t('settings.user.passwordTitle') }}</div>
      <div class="row q-col-gutter-md">
        <div class="col-12 col-md-4">
          <q-input
            v-model="passwordForm.current_password"
            :label="t('settings.user.currentPasswordLabel')"
            type="password"
            outlined
            dense
          />
        </div>
        <div class="col-12 col-md-4">
          <q-input
            v-model="passwordForm.new_password"
            :label="t('settings.user.newPasswordLabel')"
            type="password"
            outlined
            dense
          />
        </div>
        <div class="col-12 col-md-4">
          <q-input
            v-model="passwordForm.confirm_password"
            :label="t('settings.user.confirmPasswordLabel')"
            type="password"
            outlined
            dense
          />
        </div>
      </div>

      <div v-if="totpStatus?.enabled" class="q-mt-md">
        <q-tabs v-model="passwordTwoFactorMode" dense align="left" narrow-indicator>
          <q-tab name="totp" icon="password" :label="t('settings.user.twoFactorCodeTab')" />
          <q-tab name="backup" icon="vpn_key" :label="t('settings.user.backupCodeTab')" />
        </q-tabs>

        <q-input
          v-if="passwordTwoFactorMode === 'totp'"
          v-model="passwordTwoFactorCode"
          :label="t('settings.user.twoFactorCodeLabel')"
          outlined
          dense
          class="q-mt-sm"
          autocomplete="one-time-code"
          inputmode="numeric"
          maxlength="8"
        />

        <q-input
          v-else
          v-model="passwordBackupCode"
          :label="t('settings.user.backupCodeLabel')"
          outlined
          dense
          class="q-mt-sm"
          autocomplete="one-time-code"
        />
      </div>
    </q-card-section>

    <q-card-actions align="right">
      <q-btn
        color="primary"
        unelevated
        :label="t('settings.user.updatePasswordButton')"
        :loading="isSavingPassword"
        @click="savePassword"
      />
    </q-card-actions>

    <q-dialog v-model="isSetupDialogOpen" persistent>
      <q-card class="two-factor-dialog">
        <q-card-section class="row items-center justify-between">
          <div>
            <div class="text-h6">{{ t('settings.user.setupDialogTitle') }}</div>
            <div class="text-caption" style="color: var(--text-muted)">
              {{ t('settings.user.setupDialogSubtitle') }}
            </div>
          </div>
          <q-btn flat round dense icon="close" @click="cancelTotpSetup" />
        </q-card-section>

        <q-separator />

        <q-card-section v-if="totpSetup" class="q-gutter-md">
          <div class="row q-col-gutter-md items-center">
            <div class="col-12 col-sm-auto">
              <q-img :src="totpSetup.qr_data_uri" class="q-ml-md q-mt-md totp-qr" fit="contain" />
            </div>
            <div class="col-12 col-sm">
              <q-input
                :model-value="totpSetup.manual_entry_key"
                :label="t('settings.user.manualKeyLabel')"
                readonly
                outlined
                dense
              >
                <template #append>
                  <q-btn
                    flat
                    round
                    dense
                    icon="content_copy"
                    @click="copyToClipboard(totpSetup.manual_entry_key)"
                  />
                </template>
              </q-input>
              <q-input
                v-model="setupVerificationCode"
                :label="t('settings.user.confirmTwoFactorCodeLabel')"
                outlined
                dense
                class="q-mt-md"
                autocomplete="one-time-code"
                inputmode="numeric"
                maxlength="8"
              />
            </div>
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="t('common.cancel')" @click="cancelTotpSetup" />
          <q-btn
            color="primary"
            unelevated
            :label="t('settings.user.enableTwoFactorButton')"
            :loading="isLoadingTotp"
            @click="enableTotp"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <q-dialog v-model="isBackupCodesDialogOpen" persistent>
      <q-card class="two-factor-dialog">
        <q-card-section>
          <div class="text-h6">{{ t('settings.user.backupCodesDialogTitle') }}</div>
          <div class="text-caption" style="color: var(--text-muted)">
            {{ t('settings.user.backupCodesDialogSubtitle') }}
          </div>
        </q-card-section>

        <q-card-section>
          <div class="backup-code-grid">
            <div v-for="code in backupCodes" :key="code" class="backup-code">
              {{ code }}
            </div>
          </div>
        </q-card-section>

        <q-card-actions align="right">
          <q-btn
            flat
            icon="content_copy"
            :label="t('common.copyToClipboard')"
            @click="copyBackupCodes"
          />
          <q-btn
            flat
            icon="download"
            :label="t('settings.user.downloadBackupCodes')"
            @click="downloadBackupCodes"
          />
          <q-btn
            color="primary"
            unelevated
            :label="t('common.close')"
            @click="isBackupCodesDialogOpen = false"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <q-dialog v-model="isDisableTotpDialogOpen">
      <q-card class="two-factor-dialog">
        <q-card-section>
          <div class="text-h6">{{ t('settings.user.disableDialogTitle') }}</div>
          <div class="text-caption" style="color: var(--text-muted)">
            {{ t('settings.user.disableDialogSubtitle') }}
          </div>
        </q-card-section>

        <q-card-section class="q-gutter-md">
          <q-input
            v-model="disableTotpPassword"
            :label="t('settings.user.currentPasswordLabel')"
            type="password"
            outlined
            dense
          />

          <q-tabs v-model="disableTwoFactorMode" dense align="left" narrow-indicator>
            <q-tab name="totp" icon="password" :label="t('settings.user.twoFactorCodeTab')" />
            <q-tab name="backup" icon="vpn_key" :label="t('settings.user.backupCodeTab')" />
          </q-tabs>

          <q-input
            v-if="disableTwoFactorMode === 'totp'"
            v-model="disableTotpCode"
            :label="t('settings.user.twoFactorCodeLabel')"
            outlined
            dense
            autocomplete="one-time-code"
            inputmode="numeric"
            maxlength="8"
          />
          <q-input
            v-else
            v-model="disableBackupCode"
            :label="t('settings.user.backupCodeLabel')"
            outlined
            dense
            autocomplete="one-time-code"
          />
        </q-card-section>

        <q-card-actions align="right">
          <q-btn flat :label="t('common.cancel')" v-close-popup />
          <q-btn
            color="negative"
            unelevated
            :label="t('settings.user.disableTwoFactorButton')"
            :loading="isLoadingTotp"
            @click="disableTotp"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-card>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from 'src/stores/auth';
import { UsersService } from 'src/services/users';
import { useUI } from 'src/composables/useUI';
import { useCopyClipboard } from 'src/composables/useCopyClipboard';
import type { TotpSetup, TotpStatus, UpdateUserPasswordDto, UserSettings } from 'src/types/user';

const props = defineProps<{
  user: UserSettings | null;
}>();

const emit = defineEmits<{
  updated: [value: UserSettings];
}>();

const authStore = useAuthStore();
const { t } = useI18n();
const { notify, notifyError } = useUI();
const { copyToClipboard } = useCopyClipboard();

const isSavingProfile = ref(false);
const isSavingPassword = ref(false);
const isLoadingTotp = ref(false);
const isSetupDialogOpen = ref(false);
const isBackupCodesDialogOpen = ref(false);
const isDisableTotpDialogOpen = ref(false);
const totpSetup = ref<TotpSetup | null>(null);
const totpStatus = ref<TotpStatus | null>(null);
const backupCodes = ref<string[]>([]);
const backupCodesFile = ref('');
const setupVerificationCode = ref('');
const passwordTwoFactorMode = ref<'totp' | 'backup'>('totp');
const passwordTwoFactorCode = ref('');
const passwordBackupCode = ref('');
const disableTwoFactorMode = ref<'totp' | 'backup'>('totp');
const disableTotpPassword = ref('');
const disableTotpCode = ref('');
const disableBackupCode = ref('');

const profileForm = reactive({
  name: '',
  email: '',
});

const passwordForm = reactive<UpdateUserPasswordDto>({
  current_password: '',
  new_password: '',
  confirm_password: '',
});

async function saveProfile() {
  if (!profileForm.name.trim()) {
    notify(t('settings.user.validation.nameRequired'), 'warning');
    return;
  }

  if (!profileForm.email.trim()) {
    notify(t('settings.user.validation.emailRequired'), 'warning');
    return;
  }

  isSavingProfile.value = true;

  try {
    const updated = await UsersService.updateProfile({
      name: profileForm.name,
      email: profileForm.email,
    });

    emit('updated', updated);

    authStore.setAuthenticated(true, {
      id: updated.id,
      name: updated.name,
      email: updated.email,
      role: updated.role,
      status: updated.status,
      created_at: updated.created_at,
      updated_at: updated.updated_at,
    });

    notify(t('settings.user.profileSuccess'), 'positive');
  } catch (error: unknown) {
    notifyError(t('settings.user.profileError'), error);
  } finally {
    isSavingProfile.value = false;
  }
}

async function savePassword() {
  if (
    !passwordForm.current_password.trim() ||
    !passwordForm.new_password.trim() ||
    !passwordForm.confirm_password.trim()
  ) {
    notify(t('settings.user.validation.passwordFieldsRequired'), 'warning');
    return;
  }

  if (passwordForm.new_password !== passwordForm.confirm_password) {
    notify(t('settings.user.validation.passwordMismatch'), 'warning');
    return;
  }

  if (passwordForm.new_password.length < 12) {
    notify(t('settings.user.validation.passwordMin'), 'warning');
    return;
  }

  if (totpStatus.value?.enabled) {
    const hasTotpCode =
      passwordTwoFactorMode.value === 'totp' && passwordTwoFactorCode.value.trim();
    const hasBackupCode =
      passwordTwoFactorMode.value === 'backup' && passwordBackupCode.value.trim();
    if (!hasTotpCode && !hasBackupCode) {
      notify(t('settings.user.validation.twoFactorRequired'), 'warning');
      return;
    }
  }

  isSavingPassword.value = true;

  try {
    const passwordPayload: UpdateUserPasswordDto = {
      current_password: passwordForm.current_password,
      new_password: passwordForm.new_password,
      confirm_password: passwordForm.confirm_password,
    };
    if (passwordTwoFactorMode.value === 'totp' && passwordTwoFactorCode.value.trim()) {
      passwordPayload.two_factor_auth_code = passwordTwoFactorCode.value.trim();
    }
    if (passwordTwoFactorMode.value === 'backup' && passwordBackupCode.value.trim()) {
      passwordPayload.two_factor_auth_backup_code = passwordBackupCode.value.trim();
    }

    await UsersService.updatePassword(passwordPayload);

    passwordForm.current_password = '';
    passwordForm.new_password = '';
    passwordForm.confirm_password = '';
    passwordTwoFactorCode.value = '';
    passwordBackupCode.value = '';
    await loadTotpStatus();

    notify(t('settings.user.passwordSuccess'), 'positive');
  } catch (error: unknown) {
    notifyError(t('settings.user.passwordError'), error);
  } finally {
    isSavingPassword.value = false;
  }
}

async function loadTotpStatus() {
  try {
    totpStatus.value = await UsersService.getTotpStatus();
  } catch (error: unknown) {
    notifyError(t('settings.user.twoFactorLoadError'), error);
  }
}

async function startTotpSetup() {
  isLoadingTotp.value = true;

  try {
    totpSetup.value = await UsersService.startTotpSetup();
    setupVerificationCode.value = '';
    isSetupDialogOpen.value = true;
    await loadTotpStatus();
  } catch (error: unknown) {
    notifyError(t('settings.user.twoFactorSetupError'), error);
  } finally {
    isLoadingTotp.value = false;
  }
}

async function enableTotp() {
  if (!setupVerificationCode.value.trim()) {
    notify(t('settings.user.validation.twoFactorRequired'), 'warning');
    return;
  }

  isLoadingTotp.value = true;

  try {
    const enabled = await UsersService.enableTotp({ code: setupVerificationCode.value.trim() });
    totpStatus.value = enabled.status;
    backupCodes.value = enabled.backup_codes;
    backupCodesFile.value = enabled.backup_codes_file;
    isSetupDialogOpen.value = false;
    isBackupCodesDialogOpen.value = true;
    updateParentTwoFactor(true);
    notify(t('settings.user.twoFactorEnabledSuccess'), 'positive');
  } catch (error: unknown) {
    notifyError(t('settings.user.twoFactorEnableError'), error);
  } finally {
    isLoadingTotp.value = false;
  }
}

async function cancelTotpSetup() {
  isSetupDialogOpen.value = false;
  totpSetup.value = null;
  setupVerificationCode.value = '';

  try {
    totpStatus.value = await UsersService.cancelTotpSetup();
  } catch (error: unknown) {
    notifyError(t('settings.user.twoFactorCancelError'), error);
  }
}

async function disableTotp() {
  const selectedCode =
    disableTwoFactorMode.value === 'totp'
      ? disableTotpCode.value.trim()
      : disableBackupCode.value.trim();

  if (!disableTotpPassword.value.trim() || !selectedCode) {
    notify(t('settings.user.validation.disableTwoFactorRequired'), 'warning');
    return;
  }

  isLoadingTotp.value = true;

  try {
    const disablePayload = {
      password: disableTotpPassword.value,
    };
    const payload =
      disableTwoFactorMode.value === 'totp'
        ? { ...disablePayload, two_factor_auth_code: selectedCode }
        : { ...disablePayload, two_factor_auth_backup_code: selectedCode };

    totpStatus.value = await UsersService.disableTotp(payload);
    disableTotpPassword.value = '';
    disableTotpCode.value = '';
    disableBackupCode.value = '';
    isDisableTotpDialogOpen.value = false;
    updateParentTwoFactor(false);
    notify(t('settings.user.twoFactorDisabledSuccess'), 'positive');
  } catch (error: unknown) {
    notifyError(t('settings.user.twoFactorDisableError'), error);
  } finally {
    isLoadingTotp.value = false;
  }
}

function copyBackupCodes() {
  copyToClipboard(backupCodes.value.join('\n'));
  notify(t('common.copied'), 'positive');
}

function downloadBackupCodes() {
  const blob = new Blob([backupCodesFile.value], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = 'horbyta-2fa-backup-codes.txt';
  link.click();
  URL.revokeObjectURL(url);
}

function updateParentTwoFactor(enabled: boolean) {
  if (!props.user) return;
  emit('updated', {
    ...props.user,
    two_factor_auth_enabled: enabled,
  });
}

watch(
  () => props.user,
  (value) => {
    if (!value) return;

    profileForm.name = value.name;
    profileForm.email = value.email;
    totpStatus.value = {
      enabled: value.two_factor_auth_enabled,
      setup_pending: false,
      backup_codes_remaining: totpStatus.value?.backup_codes_remaining ?? 0,
    };
    void loadTotpStatus();
  },
  { immediate: true },
);
</script>

<style scoped>
.two-factor-dialog {
  width: 100%;
  max-width: 640px;
}

.totp-qr {
  width: 180px;
  max-width: 100%;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--card-bg);
}

.backup-code-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 8px;
}

.backup-code {
  padding: 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-hover);
  font-family: monospace;
  text-align: center;
}
</style>
