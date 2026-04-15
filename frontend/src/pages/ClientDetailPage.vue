<template>
  <q-page class="q-pa-md row items-start q-col-gutter-md">
    <div class="col-12 col-md-2"></div>
    <div class="col-12 col-md-8">
      <q-card
        flat
        bordered
        class="q-mb-md"
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-h5 text-weight-bold" style="color: var(--text-color)">
            {{
              client ? `${client.name} (${clientStatusLabel(client.status)})` : t('common.loading')
            }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="client">
            {{
              $t('clients.detail.meta', {
                id: client.id,
                created: formatDate(client.created_at),
                updated: formatDate(client.updated_at),
              })
            }}
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        class="q-mb-md"
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ $t('clients.detail.specsTitle') }}
          </div>

          <div class="row q-col-gutter-md q-mb-md" v-if="client">
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ $t('clients.detail.email') }}
              </div>
              <div style="color: var(--text-color)">{{ client.email || 'N/A' }}</div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ $t('clients.detail.phone') }}
              </div>
              <div style="color: var(--text-color)">{{ client.phone || 'N/A' }}</div>
            </div>
            <div class="col-12 col-md-4">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ $t('clients.detail.company') }}
              </div>
              <div style="color: var(--text-color)">{{ client.company || 'N/A' }}</div>
            </div>
            <div class="col-12">
              <div class="text-caption" style="color: var(--text-muted)">
                {{ $t('clients.detail.addressLine') }}
              </div>
              <div style="color: var(--text-color)">{{ client.address_line || 'N/A' }}</div>
            </div>
          </div>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        class="q-mb-md"
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm text-weight-bold" style="color: var(--q-primary)">
            {{ t('clients.detail.auth.title') }}
          </div>
          <div class="text-caption q-mb-md" style="color: var(--text-muted)">
            {{ t('clients.detail.auth.subtitle') }}
          </div>

          <div class="q-mb-md" v-if="client">
            <div class="text-caption" style="color: var(--text-muted)">
              {{ t('clients.detail.auth.currentMethod') }}
            </div>
            <div class="text-body2 text-weight-medium" style="color: var(--text-color)">
              {{ authMethodLabel(authStatus.method) }}
            </div>
            <div
              v-if="authStatus.method === 'credentials' && authStatus.email"
              class="text-caption q-mt-xs"
              style="color: var(--text-muted)"
            >
              {{ t('clients.detail.auth.currentEmail', { email: authStatus.email }) }}
            </div>
            <div
              v-if="authStatus.method === 'magic_token'"
              class="text-caption q-mt-xs"
              style="color: var(--text-muted)"
            >
              {{
                t('clients.detail.auth.currentTokenExpiry', {
                  date: formatTokenExpiration(authStatus.magic_client_token_exp),
                })
              }}
            </div>
          </div>

          <q-separator class="q-mb-md" />

          <div class="row q-col-gutter-md">
            <div class="col-12 col-md-6">
              <div
                class="text-subtitle2 text-weight-medium q-mb-sm"
                style="color: var(--text-color)"
              >
                {{ t('clients.detail.auth.credentialsTitle') }}
              </div>

              <q-form @submit.prevent="activateCredentials" class="q-gutter-sm">
                <q-input
                  v-model="credentialsForm.email"
                  type="email"
                  outlined
                  dense
                  :label="t('clients.detail.auth.credentialsEmail')"
                  autocomplete="off"
                />
                <q-input
                  v-model="credentialsForm.password"
                  type="password"
                  outlined
                  dense
                  :label="t('clients.detail.auth.credentialsPassword')"
                  autocomplete="new-password"
                />
                <q-input
                  v-model="credentialsForm.confirmPassword"
                  type="password"
                  outlined
                  dense
                  :label="t('clients.detail.auth.credentialsPasswordConfirm')"
                  autocomplete="new-password"
                />
                <div class="text-caption" style="color: var(--text-muted)">
                  {{ t('clients.detail.auth.passwordHint') }}
                </div>
                <q-btn
                  color="primary"
                  unelevated
                  type="submit"
                  :label="t('clients.detail.auth.credentialsActivate')"
                  :loading="isSavingCredentials"
                  :disable="!client"
                />
              </q-form>
            </div>

            <div class="col-12 col-md-6">
              <div
                class="text-subtitle2 text-weight-medium q-mb-sm"
                style="color: var(--text-color)"
              >
                {{ t('clients.detail.auth.magicTokenTitle') }}
              </div>
              <div class="text-caption q-mb-md" style="color: var(--text-muted)">
                {{ t('clients.detail.auth.magicTokenDescription') }}
              </div>
              <q-btn
                color="secondary"
                unelevated
                :label="t('clients.detail.auth.magicTokenGenerate')"
                :loading="isGeneratingMagicToken"
                :disable="!client"
                @click="generateMagicToken"
              />
            </div>
          </div>

          <q-banner
            v-if="magicTokenPayload"
            class="q-mt-md rounded-borders"
            style="background-color: var(--q-dark-page); border: 1px solid var(--border-color)"
          >
            <div class="text-subtitle2 text-weight-medium q-mb-sm" style="color: var(--text-color)">
              {{ t('clients.detail.auth.generatedTitle') }}
            </div>

            <div class="text-caption" style="color: var(--text-muted)">
              {{ t('clients.detail.auth.loginEndpoint') }}
            </div>
            <div class="text-body2 q-mb-sm" style="color: var(--text-color); word-break: break-all">
              {{ clientFrontendLoginEndpoint }}
            </div>
            <q-btn
              dense
              flat
              color="primary"
              icon="content_copy"
              :label="t('clients.detail.auth.copyEndpoint')"
              @click="
                copyToClipboard(
                  clientFrontendLoginEndpoint,
                  t('clients.detail.auth.copyEndpointSuccess'),
                )
              "
            />

            <div class="text-caption q-mt-md" style="color: var(--text-muted)">
              {{ t('clients.detail.auth.generatedToken') }}
            </div>
            <div class="text-body2" style="color: var(--text-color); word-break: break-all">
              {{ magicTokenPayload.magic_client_token }}
            </div>
            <div class="text-caption q-mt-xs" style="color: var(--text-muted)">
              {{
                t('clients.detail.auth.generatedTokenExpiry', {
                  date: formatTokenExpiration(magicTokenPayload.magic_client_token_exp),
                })
              }}
            </div>
            <q-btn
              dense
              flat
              color="primary"
              icon="content_copy"
              :label="t('clients.detail.auth.copyToken')"
              @click="
                copyToClipboard(
                  magicTokenPayload.magic_client_token,
                  t('clients.detail.auth.copyTokenSuccess'),
                )
              "
            />
          </q-banner>
        </q-card-section>
      </q-card>

      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-card-section>
          <div class="text-subtitle1 q-mb-md text-weight-bold" style="color: var(--q-primary)">
            {{ $t('clients.detail.notesTitle') }}
          </div>
          <div class="q-pa-md rounded-borders" style="border: 1px solid var(--border-color)">
            <MarkdownRenderer :model-value="client?.notes || $t('clients.detail.emptyNotes')" />
          </div>
        </q-card-section>
      </q-card>
    </div>

    <div class="col-12 col-md-2" style="position: sticky; top: 80px">
      <q-card
        flat
        bordered
        style="background-color: var(--card-bg); border-color: var(--border-color)"
      >
        <q-list separator>
          <q-item clickable v-ripple @click="goBack" class="q-pa-md">
            <q-item-section avatar>
              <q-icon name="arrow_back" style="color: var(--text-color)" />
            </q-item-section>
            <q-item-section style="color: var(--text-color)">{{
              $t('clients.detail.action.back')
            }}</q-item-section>
          </q-item>

          <q-item clickable v-ripple class="text-primary q-pa-md" @click="goToEdit" v-if="client">
            <q-item-section avatar>
              <q-icon name="edit" color="primary" />
            </q-item-section>
            <q-item-section>{{ $t('clients.detail.action.edit') }}</q-item-section>
          </q-item>

          <q-item
            clickable
            v-ripple
            class="text-negative q-pa-md"
            @click="removeClient"
            v-if="client"
          >
            <q-item-section avatar>
              <q-icon name="delete" color="negative" />
            </q-item-section>
            <q-item-section>{{ $t('clients.detail.action.delete') }}</q-item-section>
          </q-item>
        </q-list>
      </q-card>
    </div>
  </q-page>
</template>
<script setup lang="ts">
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useRoute, useRouter } from 'vue-router';
import { useUI } from 'src/composables/useUI';
import { useI18n } from 'vue-i18n';
import { ClientsService } from 'src/services/clients';
import type { ClientAuthMethod, ClientAuthStatusDto } from 'src/types/clients';
import { computed, onMounted, reactive, ref } from 'vue';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';

interface GeneratedMagicToken {
  magic_client_token: string;
  magic_client_token_exp: number;
}

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmAction, confirmDelete, showLoading, hideLoading } = useUI();
const userPreferencesStore = useUserPreferencesStore();
type ClientDetailViewDto = Awaited<ReturnType<typeof ClientsService.getById>>;

const client = ref<ClientDetailViewDto | null>(null);
const authStatus = ref<ClientAuthStatusDto>({
  method: 'none',
  email: null,
  magic_client_token_exp: null,
});
const magicTokenPayload = ref<GeneratedMagicToken | null>(null);
const isSavingCredentials = ref(false);
const isGeneratingMagicToken = ref(false);
const clientId = String(route.params.id || '');
const credentialsForm = reactive({
  email: '',
  password: '',
  confirmPassword: '',
});

const formatDate = (value: string, fallback = t('common.empty')) =>
  formatDateByPreferences(value, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    fallback,
  });

const clientFrontendLoginEndpoint = computed(
  () => `${window.location.origin}${router.resolve({ name: 'client-login' }).href}`,
);

const loadClientAuthStatus = async (notifyFailure = true) => {
  try {
    authStatus.value = await ClientsService.getAuthStatus(clientId);
  } catch (err: unknown) {
    authStatus.value = {
      method: 'none',
      email: null,
      magic_client_token_exp: null,
    };
    if (notifyFailure) {
      notifyError(t('clients.detail.auth.loadStatusError'), err);
    }
  }
};

const loadClient = async () => {
  showLoading(t('clients.detail.loadClient'));

  try {
    client.value = await ClientsService.getById(clientId);
    credentialsForm.email = client.value.email || '';
    await loadClientAuthStatus(false);
  } catch (err: unknown) {
    notifyError(t('clients.detail.loadError'), err);
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  void router.push('/clients');
};

const goToEdit = () => {
  if (!client.value) return;
  void router.push(`/clients/${client.value.id}/edit`);
};

const removeClient = async () => {
  if (!client.value) return;

  const confirmed = await confirmDelete(t('clients.detail.deleteTarget'));
  if (!confirmed) return;

  showLoading(t('clients.detail.deleting'));

  try {
    await ClientsService.delete(client.value.id);
    notify(t('clients.detail.deleteSuccess'), 'positive');
    void router.push('/clients');
  } catch (err: unknown) {
    notifyError(t('clients.detail.deleteError'), err);
  } finally {
    hideLoading();
  }
};

const authMethodLabel = (method: ClientAuthMethod) => {
  return t(`clients.detail.auth.method.${method}`);
};

const clientStatusLabel = (status: ClientDetailViewDto['status']) => {
  return t(`clients.detail.status.${status}`);
};

const formatTokenExpiration = (exp: number | null | undefined) => {
  if (!exp) return t('common.empty');

  return formatDateByPreferences(exp * 1000, {
    timezone: userPreferencesStore.timezone,
    dateFormat: userPreferencesStore.dateFormat,
    includeTime: true,
    fallback: t('common.empty'),
  });
};

const shouldSwitchMethod = (nextMethod: 'credentials' | 'magic_token'): Promise<boolean> => {
  const currentMethod = authStatus.value.method;
  if (currentMethod === 'none' || currentMethod === nextMethod) {
    return Promise.resolve(true);
  }

  if (nextMethod === 'credentials') {
    return confirmAction({
      title: t('clients.detail.auth.switchToCredentialsTitle'),
      message: t('clients.detail.auth.switchToCredentialsMessage'),
      okLabel: t('clients.detail.auth.switchAction'),
      okColor: 'warning',
    });
  }

  return confirmAction({
    title: t('clients.detail.auth.switchToMagicTitle'),
    message: t('clients.detail.auth.switchToMagicMessage'),
    okLabel: t('clients.detail.auth.switchAction'),
    okColor: 'warning',
  });
};

const activateCredentials = async () => {
  if (!client.value) return;

  if (!credentialsForm.email.trim() || !credentialsForm.password) {
    notify(t('clients.detail.auth.credentialsRequired'), 'warning');
    return;
  }

  if (credentialsForm.password !== credentialsForm.confirmPassword) {
    notify(t('clients.detail.auth.passwordMismatch'), 'warning');
    return;
  }

  const canContinue = await shouldSwitchMethod('credentials');
  if (!canContinue) return;

  isSavingCredentials.value = true;
  try {
    authStatus.value = await ClientsService.activateCredentials(client.value.id, {
      email: credentialsForm.email.trim(),
      password: credentialsForm.password,
    });
    magicTokenPayload.value = null;
    credentialsForm.password = '';
    credentialsForm.confirmPassword = '';
    notify(t('clients.detail.auth.credentialsSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('clients.detail.auth.credentialsError'), err);
  } finally {
    isSavingCredentials.value = false;
  }
};

const generateMagicToken = async () => {
  if (!client.value) return;

  const canContinue = await shouldSwitchMethod('magic_token');
  if (!canContinue) return;

  isGeneratingMagicToken.value = true;
  try {
    magicTokenPayload.value = await ClientsService.activateMagicToken(client.value.id);
    await loadClientAuthStatus(false);
    notify(t('clients.detail.auth.magicTokenSuccess'), 'positive');
  } catch (err: unknown) {
    notifyError(t('clients.detail.auth.magicTokenError'), err);
  } finally {
    isGeneratingMagicToken.value = false;
  }
};

const copyToClipboard = async (value: string, successMessage: string) => {
  try {
    await navigator.clipboard.writeText(value);
    notify(successMessage, 'positive');
  } catch (err: unknown) {
    notifyError(t('clients.detail.auth.copyError'), err);
  }
};

onMounted(() => {
  void loadClient();
});
</script>
