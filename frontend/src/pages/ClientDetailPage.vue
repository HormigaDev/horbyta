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
              client?.name + ` (${$t(`clients.detail.status.${client?.status}`)})` ||
              $t('common.loading')
            }}
          </div>
          <div class="text-caption" style="color: var(--text-muted)" v-if="client">
            {{
              $t('clients.detail.meta', {
                id: client.id,
                created: new Date(client.created_at).toLocaleDateString(),
                updated: new Date(client.updated_at).toLocaleDateString(),
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
import type { Client } from 'src/types/clients';
import { onMounted, ref } from 'vue';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, confirmDelete, showLoading, hideLoading } = useUI();

const client = ref<Client | null>(null);
const clientId = String(route.params.id || '');

const loadClient = async () => {
  showLoading(t('clients.detail.loadClient'));

  try {
    client.value = await ClientsService.getById(clientId);
  } catch (err: unknown) {
    notifyError(t('clients.detail.loadError'), err);
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  router.push('/clients');
};

const goToEdit = () => {
  if (!client.value) return;
  router.push(`/clients/${client.value.id}/edit`);
};

const removeClient = async () => {
  if (!client.value) return;

  const confirmed = await confirmDelete(t('clients.detail.deleteTarget'));
  if (!confirmed) return;

  showLoading(t('clients.detail.deleting'));

  try {
    await ClientsService.delete(client.value.id);
    notify(t('clients.detail.deleteSuccess'), 'positive');
    router.push('/clients');
  } catch (err: unknown) {
    notifyError(t('clients.detail.deleteError'), err);
  } finally {
    hideLoading();
  }
};

onMounted(() => {
  loadClient();
});
</script>
