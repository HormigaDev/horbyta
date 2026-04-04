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
  </q-card>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useAuthStore } from 'src/stores/auth';
import { UsersService } from 'src/services/users';
import { useUI } from 'src/composables/useUI';
import type { UpdateUserPasswordDto, UserSettings } from 'src/types/user';

const props = defineProps<{
  user: UserSettings | null;
}>();

const emit = defineEmits<{
  updated: [value: UserSettings];
}>();

const authStore = useAuthStore();
const { t } = useI18n();
const { notify, notifyError } = useUI();

const isSavingProfile = ref(false);
const isSavingPassword = ref(false);

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

  if (passwordForm.new_password.length < 8) {
    notify(t('settings.user.validation.passwordMin'), 'warning');
    return;
  }

  isSavingPassword.value = true;

  try {
    await UsersService.updatePassword({
      current_password: passwordForm.current_password,
      new_password: passwordForm.new_password,
      confirm_password: passwordForm.confirm_password,
    });

    passwordForm.current_password = '';
    passwordForm.new_password = '';
    passwordForm.confirm_password = '';

    notify(t('settings.user.passwordSuccess'), 'positive');
  } catch (error: unknown) {
    notifyError(t('settings.user.passwordError'), error);
  } finally {
    isSavingPassword.value = false;
  }
}

watch(
  () => props.user,
  (value) => {
    if (!value) return;

    profileForm.name = value.name;
    profileForm.email = value.email;
  },
  { immediate: true },
);
</script>
