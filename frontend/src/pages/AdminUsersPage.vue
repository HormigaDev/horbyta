<template>
  <q-page class="q-pa-md flex column">
    <div class="admin-users-header q-mb-md">
      <div class="row items-center justify-between q-col-gutter-md">
        <div class="col-12 col-md">
          <div class="row items-center q-gutter-sm q-mb-xs">
            <q-icon name="admin_panel_settings" color="warning" size="md" />
            <div class="text-h5 text-weight-bold">{{ t('adminUsers.page.title') }}</div>
            <q-badge color="warning" text-color="dark">
              {{ t('adminUsers.page.adminMode') }}
            </q-badge>
          </div>
          <div class="text-caption" style="color: var(--text-muted)">
            {{ t('adminUsers.page.subtitle') }}
          </div>
        </div>
      </div>
      <q-banner rounded class="admin-users-warning q-mt-md">
        <template #avatar>
          <q-icon name="warning" color="negative" />
        </template>
        <div class="text-subtitle2">{{ t('adminUsers.warning.title') }}</div>
        <div class="text-caption">{{ t('adminUsers.warning.message') }}</div>
      </q-banner>
    </div>

    <DataTable
      class="col"
      table-id="admin_users_table"
      :title="t('adminUsers.table.title')"
      icon="manage_accounts"
      :columns="getUserColumns($t)"
      :rows="items"
      :total-items="total"
      :loading="isLoading"
      :filter-schema="USER_FILTER_SCHEMA"
      hide-view
      @request="onRequest"
      @selection-change="selectedRows = $event"
      @create="openCreateDialog"
      @edit="openEditDialog"
      @delete="openDeleteDialog"
      @refresh="fetch"
    />

    <q-dialog v-model="userDialogOpen" persistent>
      <q-card class="admin-user-dialog">
        <q-card-section class="row items-start justify-between">
          <div>
            <div class="text-h6">
              {{
                isEditing ? t('adminUsers.dialog.editTitle') : t('adminUsers.dialog.createTitle')
              }}
            </div>
            <div class="text-caption" style="color: var(--text-muted)">
              {{ t('adminUsers.dialog.subtitle') }}
            </div>
          </div>
          <q-icon name="admin_panel_settings" color="warning" size="sm" />
        </q-card-section>

        <q-separator />

        <q-card-section>
          <q-form class="q-col-gutter-md" @submit.prevent="saveUser">
            <q-input
              v-model="form.name"
              outlined
              dense
              :label="t('adminUsers.dialog.nameLabel')"
              autocomplete="off"
            />
            <q-input
              v-model="form.email"
              outlined
              dense
              type="email"
              :label="t('adminUsers.dialog.emailLabel')"
              autocomplete="off"
            />
            <q-input
              v-model="form.password"
              outlined
              dense
              type="password"
              :label="
                isEditing
                  ? t('adminUsers.dialog.passwordOptionalLabel')
                  : t('adminUsers.dialog.passwordLabel')
              "
              autocomplete="new-password"
            />
            <div class="row q-col-gutter-md">
              <div class="col-12 col-sm-6">
                <q-select
                  v-model="form.role"
                  outlined
                  dense
                  emit-value
                  map-options
                  :options="roleOptions"
                  :label="t('adminUsers.dialog.roleLabel')"
                />
              </div>
              <div class="col-12 col-sm-6">
                <q-select
                  v-model="form.status"
                  outlined
                  dense
                  emit-value
                  map-options
                  :options="statusOptions"
                  :label="t('adminUsers.dialog.statusLabel')"
                />
              </div>
            </div>
          </q-form>
        </q-card-section>

        <q-card-actions align="right" class="q-pa-md">
          <q-btn flat :label="t('common.cancel')" :disable="isSaving" @click="closeUserDialog" />
          <q-btn
            unelevated
            color="primary"
            icon="save"
            :label="t('common.save')"
            :loading="isSaving"
            @click="saveUser"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>

    <q-dialog v-model="deleteDialogOpen" persistent>
      <q-card class="admin-delete-dialog">
        <q-card-section class="row items-start q-col-gutter-md">
          <div class="col-auto">
            <q-avatar color="negative" text-color="white" icon="delete_forever" />
          </div>
          <div class="col">
            <div class="text-h6">{{ t('adminUsers.delete.title') }}</div>
            <div class="text-caption" style="color: var(--text-muted)">
              {{ t('adminUsers.delete.message', { email: deletingUser?.email || '' }) }}
            </div>
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-input
            v-model="deleteConfirmation"
            outlined
            dense
            autocomplete="off"
            :label="t('adminUsers.delete.confirmLabel')"
            :hint="t('adminUsers.delete.confirmHint')"
            :error="deleteConfirmation.length > 0 && !canDelete"
            :error-message="t('adminUsers.delete.confirmError')"
            @paste.prevent
            @drop.prevent
            @beforeinput="preventNonManualDeleteConfirmInput"
          />
        </q-card-section>

        <q-card-actions align="right" class="q-pa-md">
          <q-btn
            flat
            :label="t('common.cancel')"
            :disable="isDeleting"
            @click="closeDeleteDialog"
          />
          <q-btn
            unelevated
            color="negative"
            icon="delete"
            :label="t('common.delete')"
            :disable="!canDelete"
            :loading="isDeleting"
            @click="deleteUser"
          />
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import DataTable from 'src/components/DataTable.vue';
import { useCrudListHandlers } from 'src/composables/useCrudListHandlers';
import { useQuery } from 'src/composables/useQuery';
import { useUI } from 'src/composables/useUI';
import { getUserColumns } from 'src/data/columns';
import { UsersService } from 'src/services/users';
import { USER_FILTER_SCHEMA } from 'src/types/filterSchemas';
import type {
  AdminCreateUserDto,
  AdminUpdateUserDto,
  AdminUserSummary,
  UserRole,
  UserStatus,
} from 'src/types/user';

const { t } = useI18n();
const { notify, notifyError } = useUI();

const { items, total, isLoading, query, fetch } = useQuery(
  UsersService.search,
  USER_FILTER_SCHEMA,
  10,
  'created_at',
  false,
);
const { onRequest } = useCrudListHandlers({ query, fetch });

const selectedRows = ref<AdminUserSummary[]>([]);
const userDialogOpen = ref(false);
const deleteDialogOpen = ref(false);
const isSaving = ref(false);
const isDeleting = ref(false);
const editingId = ref<string | null>(null);
const deletingUser = ref<AdminUserSummary | null>(null);
const deleteConfirmation = ref('');

const form = reactive({
  name: '',
  email: '',
  password: '',
  role: 'user' as UserRole,
  status: 'active' as UserStatus,
});

const isEditing = computed(() => Boolean(editingId.value));
const canDelete = computed(
  () => Boolean(deletingUser.value) && deleteConfirmation.value === deletingUser.value?.email,
);

const roleOptions = computed(() => [
  { label: t('adminUsers.role.user'), value: 'user' },
  { label: t('adminUsers.role.admin'), value: 'admin' },
]);

const statusOptions = computed(() => [
  { label: t('adminUsers.status.active'), value: 'active' },
  { label: t('adminUsers.status.inactive'), value: 'inactive' },
  { label: t('adminUsers.status.blocked'), value: 'blocked' },
]);

function resetForm() {
  form.name = '';
  form.email = '';
  form.password = '';
  form.role = 'user';
  form.status = 'active';
  editingId.value = null;
}

function openCreateDialog() {
  resetForm();
  userDialogOpen.value = true;
}

function openEditDialog(ids: string[]) {
  if (ids.length !== 1) {
    notify(t('adminUsers.messages.selectOneToEdit'), 'warning');
    return;
  }

  const row = selectedRows.value.find((user) => user.id === ids[0]);
  if (!row) return;

  editingId.value = row.id;
  form.name = row.name;
  form.email = row.email;
  form.password = '';
  form.role = row.role;
  form.status = row.status;
  userDialogOpen.value = true;
}

function closeUserDialog() {
  userDialogOpen.value = false;
  resetForm();
}

function validateForm(): boolean {
  if (!form.name.trim() || !form.email.trim()) {
    notify(t('adminUsers.messages.requiredFields'), 'warning');
    return false;
  }

  if (!isEditing.value && !form.password.trim()) {
    notify(t('adminUsers.messages.passwordRequired'), 'warning');
    return false;
  }

  return true;
}

async function saveUser() {
  if (!validateForm()) return;

  isSaving.value = true;
  try {
    if (editingId.value) {
      const payload: AdminUpdateUserDto = {
        name: form.name.trim(),
        email: form.email.trim(),
        role: form.role,
        status: form.status,
      };
      if (form.password.trim()) {
        payload.password = form.password;
      }
      await UsersService.adminUpdate(editingId.value, payload);
      notify(t('adminUsers.messages.updateSuccess'), 'positive');
    } else {
      const payload: AdminCreateUserDto = {
        name: form.name.trim(),
        email: form.email.trim(),
        password: form.password,
        role: form.role,
        status: form.status,
      };
      await UsersService.adminCreate(payload);
      notify(t('adminUsers.messages.createSuccess'), 'positive');
    }

    closeUserDialog();
    await fetch();
  } catch (error: unknown) {
    notifyError(t('adminUsers.messages.saveError'), error);
  } finally {
    isSaving.value = false;
  }
}

function openDeleteDialog(ids: string[]) {
  if (ids.length !== 1) {
    notify(t('adminUsers.messages.selectOneToDelete'), 'warning');
    return;
  }

  const row = selectedRows.value.find((user) => user.id === ids[0]);
  if (!row) return;

  deletingUser.value = row;
  deleteConfirmation.value = '';
  deleteDialogOpen.value = true;
}

function closeDeleteDialog() {
  deleteDialogOpen.value = false;
  deletingUser.value = null;
  deleteConfirmation.value = '';
}

function preventNonManualDeleteConfirmInput(event: Event) {
  const inputEvent = event as InputEvent;
  if (inputEvent.inputType === 'insertFromPaste' || inputEvent.inputType === 'insertFromDrop') {
    event.preventDefault();
  }
}

async function deleteUser() {
  if (!deletingUser.value || !canDelete.value) return;

  isDeleting.value = true;
  try {
    await UsersService.adminDelete(deletingUser.value.id);
    notify(t('adminUsers.messages.deleteSuccess'), 'positive');
    closeDeleteDialog();
    await fetch();
  } catch (error: unknown) {
    notifyError(t('adminUsers.messages.deleteError'), error);
  } finally {
    isDeleting.value = false;
  }
}
</script>

<style scoped>
.admin-users-header,
.admin-users-warning {
  border: 1px solid var(--border-color);
  background: var(--card-bg);
}

.admin-users-header {
  border-radius: 8px;
  padding: 16px;
}

.admin-users-warning {
  color: var(--text-color);
}

.admin-user-dialog,
.admin-delete-dialog {
  width: 100%;
  max-width: 560px;
  background: var(--card-bg);
  color: var(--text-color);
}
</style>
