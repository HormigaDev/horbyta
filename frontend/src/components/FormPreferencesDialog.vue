<template>
  <q-dialog v-model="dialogModel" persistent>
    <q-card style="min-width: 420px; max-width: 95vw; max-height: 80vh" class="column">
      <q-card-section class="row items-center q-pb-none">
        <div class="text-h6 flex items-center">
          <q-icon name="tune" class="q-mr-sm" />
          {{ t('formPreferences.title') }}
        </div>
        <q-space />
        <q-btn icon="close" flat round dense v-close-popup />
      </q-card-section>

      <q-card-section class="col overflow-auto">
        <q-inner-loading :showing="isLoading">
          <q-spinner color="primary" size="28px" />
        </q-inner-loading>

        <div v-if="!isLoading && preferences.length === 0" class="text-center q-py-md">
          <q-icon name="bookmark_border" size="40px" style="color: var(--text-muted)" />
          <div class="text-caption q-mt-sm" style="color: var(--text-muted)">
            {{ t('formPreferences.empty') }}
          </div>
        </div>

        <q-list v-if="!isLoading && preferences.length > 0" separator>
          <q-item v-for="pref in preferences" :key="pref.id" class="q-px-none">
            <q-item-section side>
              <q-btn
                flat
                round
                dense
                size="sm"
                :icon="defaultId === pref.id ? 'star' : 'star_border'"
                :color="defaultId === pref.id ? 'amber' : 'grey'"
                :loading="isSaving"
                @click="toggleDefault(pref.id)"
              >
                <q-tooltip>{{
                  defaultId === pref.id
                    ? t('formPreferences.unsetDefault')
                    : t('formPreferences.setDefault')
                }}</q-tooltip>
              </q-btn>
            </q-item-section>

            <q-item-section>
              <q-item-label>{{ pref.name }}</q-item-label>
              <q-item-label v-if="defaultId === pref.id" caption style="color: var(--text-muted)">
                {{ t('formPreferences.defaultLabel') }}
              </q-item-label>
            </q-item-section>

            <q-item-section side>
              <div class="row no-wrap q-gutter-xs">
                <q-btn
                  flat
                  dense
                  size="sm"
                  color="primary"
                  :label="t('formPreferences.applyAction')"
                  @click="applyPref(pref)"
                />
                <q-btn flat round dense size="sm" icon="more_vert">
                  <q-menu>
                    <q-list dense style="min-width: 130px">
                      <q-item clickable v-close-popup @click="startRename(pref)">
                        <q-item-section side><q-icon name="edit" size="sm" /></q-item-section>
                        <q-item-section>{{ t('common.rename') }}</q-item-section>
                      </q-item>
                      <q-item clickable v-close-popup @click="confirmDelete(pref)">
                        <q-item-section side
                          ><q-icon name="delete" size="sm" color="negative"
                        /></q-item-section>
                        <q-item-section class="text-negative">{{
                          t('common.delete')
                        }}</q-item-section>
                      </q-item>
                    </q-list>
                  </q-menu>
                </q-btn>
              </div>
            </q-item-section>
          </q-item>
        </q-list>
      </q-card-section>

      <q-separator />

      <q-card-actions align="between" class="q-px-md q-py-sm">
        <q-btn
          flat
          icon="add"
          color="primary"
          :label="t('formPreferences.saveCurrentAction')"
          :disable="preferences.length >= 10 || isSaving"
          @click="startSaveCurrent"
        />
        <q-btn flat :label="t('common.close')" v-close-popup />
      </q-card-actions>
    </q-card>
  </q-dialog>

  <!-- Rename / save-name dialog -->
  <q-dialog v-model="nameDialogOpen" persistent>
    <q-card style="min-width: 340px; max-width: 90vw">
      <q-card-section class="row items-center q-pb-none">
        <div class="text-h6">{{ nameDialogTitle }}</div>
        <q-space />
        <q-btn icon="close" flat round dense @click="nameDialogOpen = false" />
      </q-card-section>
      <q-card-section>
        <q-input
          v-model="nameDialogValue"
          :label="t('formPreferences.nameLabel')"
          outlined
          dense
          autofocus
          :error="nameDialogError"
          :error-message="t('formPreferences.nameRequired')"
          @keyup.enter="submitNameDialog"
        />
      </q-card-section>
      <q-card-actions align="right" class="q-px-md q-pb-md">
        <q-btn flat :label="t('common.cancel')" @click="nameDialogOpen = false" />
        <q-btn
          color="primary"
          :label="t('common.save')"
          :loading="isSaving"
          @click="submitNameDialog"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import {
  useFormPreferences,
  type FormPrefKey,
  type FormPreference,
} from 'src/composables/useFormPreferences';

interface Props {
  modelValue: boolean;
  formKey: FormPrefKey;
  currentFields: Record<string, unknown>;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'apply', fields: Record<string, unknown>): void;
}>();

const $q = useQuasar();
const { t } = useI18n();

const dialogModel = computed({
  get: () => props.modelValue,
  set: (v: boolean) => emit('update:modelValue', v),
});

const {
  isLoading,
  isSaving,
  preferences,
  defaultId,
  addPreference,
  removePreference,
  setDefault,
  renamePreference,
} = useFormPreferences(props.formKey);

const nameDialogOpen = ref(false);
const nameDialogValue = ref('');
const nameDialogError = ref(false);
const nameDialogTitle = ref('');
const nameDialogMode = ref<'save' | 'rename'>('save');
const renamingId = ref<string | null>(null);

const toggleDefault = async (id: string) => {
  await setDefault(defaultId.value === id ? null : id);
};

const applyPref = (pref: FormPreference) => {
  emit('apply', pref.fields);
  emit('update:modelValue', false);
};

const startSaveCurrent = () => {
  nameDialogMode.value = 'save';
  nameDialogTitle.value = t('formPreferences.saveCurrentTitle');
  nameDialogValue.value = '';
  nameDialogError.value = false;
  nameDialogOpen.value = true;
};

const startRename = (pref: FormPreference) => {
  nameDialogMode.value = 'rename';
  nameDialogTitle.value = t('formPreferences.renameTitle');
  nameDialogValue.value = pref.name;
  nameDialogError.value = false;
  renamingId.value = pref.id;
  nameDialogOpen.value = true;
};

const submitNameDialog = async () => {
  const trimmed = nameDialogValue.value.trim();
  if (!trimmed) {
    nameDialogError.value = true;
    return;
  }
  nameDialogError.value = false;

  if (nameDialogMode.value === 'save') {
    await addPreference(trimmed, props.currentFields);
  } else if (renamingId.value) {
    await renamePreference(renamingId.value, trimmed);
    renamingId.value = null;
  }

  nameDialogOpen.value = false;
};

const confirmDelete = (pref: FormPreference) => {
  $q.dialog({
    title: t('formPreferences.deleteTitle'),
    message: t('formPreferences.deleteMessage', { name: pref.name }),
    ok: { label: t('common.delete'), color: 'negative', unelevated: true },
    cancel: { label: t('common.cancel'), flat: true },
  }).onOk(async () => {
    await removePreference(pref.id);
  });
};
</script>
