<template>
  <q-card flat bordered>
    <q-card-section>
      <div class="text-h6">Usuario</div>
      <div class="text-caption" style="color: var(--text-muted)">
        Actualiza nombre, correo y contraseña de tu cuenta autenticada.
      </div>
    </q-card-section>

    <q-separator />

    <q-card-section class="row q-col-gutter-md">
      <div class="col-12 col-md-6">
        <q-input v-model="profileForm.name" label="Nombre" outlined dense />
      </div>
      <div class="col-12 col-md-6">
        <q-input
          v-model="profileForm.email"
          label="Correo electrónico"
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
        label="Guardar usuario"
        :loading="isSavingProfile"
        @click="saveProfile"
      />
    </q-card-actions>

    <q-separator />

    <q-card-section>
      <div class="text-subtitle1 q-mb-sm">Cambiar contraseña</div>
      <div class="row q-col-gutter-md">
        <div class="col-12 col-md-4">
          <q-input
            v-model="passwordForm.current_password"
            label="Contraseña actual"
            type="password"
            outlined
            dense
          />
        </div>
        <div class="col-12 col-md-4">
          <q-input
            v-model="passwordForm.new_password"
            label="Nueva contraseña"
            type="password"
            outlined
            dense
          />
        </div>
        <div class="col-12 col-md-4">
          <q-input
            v-model="passwordForm.confirm_password"
            label="Confirmar contraseña"
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
        label="Actualizar contraseña"
        :loading="isSavingPassword"
        @click="savePassword"
      />
    </q-card-actions>
  </q-card>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue';
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
    notify('El nombre es obligatorio', 'warning');
    return;
  }

  if (!profileForm.email.trim()) {
    notify('El correo es obligatorio', 'warning');
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

    notify('Usuario actualizado', 'positive');
  } catch (error: unknown) {
    notifyError('No se pudo actualizar el usuario', error);
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
    notify('Completa los tres campos de contraseña', 'warning');
    return;
  }

  if (passwordForm.new_password !== passwordForm.confirm_password) {
    notify('La confirmación no coincide con la nueva contraseña', 'warning');
    return;
  }

  if (passwordForm.new_password.length < 8) {
    notify('La nueva contraseña debe tener al menos 8 caracteres', 'warning');
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

    notify('Contraseña actualizada', 'positive');
  } catch (error: unknown) {
    notifyError('No se pudo actualizar la contraseña', error);
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
