<template>
  <q-page class="q-pa-md">
    <div class="padding-page">
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6 flex items-center">
            <q-icon :name="isNew ? 'person_add' : 'person_edit'" size="md" class="q-mr-md" />
            {{ isNew ? $t('clients.form.createTitle') : $t('clients.form.editTitle') }}
          </div>
          <div class="text-caption text-grey" v-if="!isNew">
            {{ $t('clients.form.editingPrefix', { id: client.id }) }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-8">
            <q-input
              outlined
              dense
              v-model="client.name"
              :label="$t('clients.form.nameLabel')"
              :maxlength="255"
            />
          </div>
          <div class="col-12 col-md-4">
            <q-select
              :disable="isNew"
              outlined
              dense
              emit-value
              map-options
              v-model="client.status"
              :label="$t('clients.form.statusLabel')"
              :options="[
                { label: $t('clients.form.statusActive'), value: 'active' },
                { label: $t('clients.form.statusInactive'), value: 'inactive' },
                { label: $t('clients.form.statusBlocked'), value: 'blocked' },
              ]"
            />
          </div>
          <div class="col-12 col-md-4">
            <q-input
              outlined
              dense
              v-model="client.email"
              :label="$t('clients.form.emailLabel')"
              :maxlength="255"
              :error="emailRuleState.error"
              :error-message="emailRuleState.message"
              @update:model-value="validateEmail(client.email)"
            />
          </div>
          <div class="col-12 col-md-4">
            <q-input
              outlined
              dense
              v-model="client.phone"
              :maxlength="20"
              :label="$t('clients.form.phoneLabel')"
              :error="phoneRuleState.error"
              :error-message="phoneRuleState.message"
              @update:model-value="validatePhone(client.phone)"
            />
          </div>
          <div class="col-12 col-md-4">
            <q-input
              outlined
              dense
              v-model="client.company"
              :label="$t('clients.form.companyLabel')"
              :maxlength="255"
            />
          </div>
          <div class="col-12">
            <q-input
              outlined
              dense
              v-model="client.address_line"
              :label="$t('clients.form.addressLineLabel')"
              :maxlength="512"
            />
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">{{ $t('clients.form.notesTitle') }}</div>
          <div class="row q-col-gutter-md" style="min-height: 50vh">
            <div class="col-12" style="display: flex; flex-direction: column">
              <MarkdownRenderer
                v-model="client.notes!"
                edit-mode
                enable-preview-toggle
                class="full-height"
              />
            </div>
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn
          flat
          round
          icon="push_pin"
          :color="pinButtonColor"
          :aria-pressed="stayOnFormAfterSave"
          @click="stayOnFormAfterSave = !stayOnFormAfterSave"
        >
          <q-tooltip>{{ t('common.keepFormOpen') }}</q-tooltip>
        </q-btn>
        <q-btn :label="$t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          :label="$t('clients.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveClient"
        />
      </div>
    </div>
  </q-page>
</template>
<script setup lang="ts">
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import { useClientRules } from 'src/composables/rules/client.rules';
import { usePinnedFormSave } from 'src/composables/usePinnedFormSave';
import { useUI } from 'src/composables/useUI';
import { ClientsService } from 'src/services/clients';
import type { ClientDetailDto, CreateClientDto, UpdateClientDto } from 'src/types/clients';
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useRoute, useRouter } from 'vue-router';

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { emailRuleState, phoneRuleState, validateEmail, validatePhone } = useClientRules();
const { notify, notifyError, showLoading, hideLoading } = useUI();

const isNew = computed(() => route.name === 'client-create');
const isSubmitting = ref(false);
const { stayOnFormAfterSave, pinButtonColor, handlePinnedSaveSuccess } = usePinnedFormSave();

const createEmptyClient = (): ClientDetailDto => ({
  id: '',
  name: '',
  status: 'active',
  email: '',
  phone: '',
  address_line: '',
  company: '',
  notes: '',
  created_at: '',
  updated_at: '',
});

const client = ref<ClientDetailDto>(createEmptyClient());

const resetClientForm = async () => {
  emailRuleState.value = { error: false, message: '' };
  phoneRuleState.value = { error: false, message: '' };

  if (isNew.value) {
    client.value = createEmptyClient();
    return;
  }

  await loadClient();
};

const loadClient = async () => {
  if (isNew.value) return;

  const clientId = route.params.id as string;
  showLoading(t('clients.form.loadingClient'));

  try {
    const data = await ClientsService.getById(clientId);
    client.value = data;
  } catch (err: unknown) {
    notifyError(t('clients.form.loadError'), err);
    router.push('/clients');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
  if (isNew.value) {
    void router.push('/clients');
  } else {
    void router.push(`/clients/${client.value.id}`);
  }
};

const saveClient = async () => {
  const hasEmailError = validateEmail(client.value.email);
  const hasPhoneError = validatePhone(client.value.phone);

  if (hasEmailError || hasPhoneError) return;

  isSubmitting.value = true;
  showLoading(t('clients.form.saving'));

  try {
    const payload: CreateClientDto = {
      name: client.value.name,
      email: client.value.email!,
      phone: client.value.phone!,
      address_line: client.value.address_line!,
      company: client.value.company!,
      notes: client.value.notes!,
    };
    client.value.email && (payload.email = client.value.email);
    client.value.phone && (payload.phone = client.value.phone);
    client.value.address_line && (payload.address_line = client.value.address_line);
    client.value.company && (payload.company = client.value.company);
    client.value.notes && (payload.notes = client.value.notes);

    if (isNew.value) {
      const created = await ClientsService.create(payload);
      notify(t('clients.form.createSuccess'), 'positive');
      await handlePinnedSaveSuccess({
        reset: resetClientForm,
        redirect: () => router.push(`/clients/${created.id}`),
      });
    } else {
      const updatedPayload: UpdateClientDto = {
        name: payload.name,
        email: payload.email!,
        status: client.value.status,
        phone: payload.phone!,
        address_line: payload.address_line!,
        company: payload.company!,
        notes: payload.notes!,
      };
      await ClientsService.update(client.value.id, updatedPayload);
      notify(t('clients.form.updateSuccess'), 'positive');
      await handlePinnedSaveSuccess({
        reset: resetClientForm,
        redirect: () => router.push(`/clients/${client.value.id}`),
      });
    }
  } catch (err: unknown) {
    notifyError(t('clients.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(() => {
  loadClient();
});
</script>
