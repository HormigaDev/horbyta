<template>
  <q-dialog v-model="dialogModel" persistent @hide="onDialogClosed">
    <q-card style="width: min(420px, calc(100vw - 32px)); max-width: 92vw">
      <q-form @submit.prevent="onSubmit">
        <q-card-section class="row items-center q-pb-none">
          <div class="text-h6 flex items-center">
            <q-icon :name="!editingId ? 'add' : 'edit'" size="md" class="q-mr-md" />
            {{ editingId ? t('tags.dialog.editTitle') : t('tags.dialog.newTitle') }}
          </div>
          <q-space />
          <q-btn
            :aria-label="t('common.close')"
            icon="close"
            flat
            round
            dense
            @click="dialogModel = false"
          />
        </q-card-section>

        <q-card-section v-if="isLoadingTag" class="q-py-xl flex flex-center">
          <q-spinner-dots color="primary" size="32px" />
        </q-card-section>

        <q-card-section v-else class="q-pt-md q-gutter-md">
          <div
            class="q-pa-sm rounded-borders"
            style="border: 1px solid var(--border-color); background-color: var(--bg-hover)"
          >
            <div class="text-caption q-mb-xs" style="color: var(--text-muted)">
              {{ t('tags.dialog.previewLabel') }}
            </div>
            <q-chip dense square :style="previewStyle">
              {{ previewName }}
            </q-chip>
          </div>

          <q-input
            v-model="nameModel"
            :label="t('tags.dialog.nameLabel')"
            autofocus
            outlined
            :error="nameRuleState.error"
            :error-message="nameRuleState.message"
            :disable="isSaving"
            @update:model-value="onNameInput"
          />

          <q-input
            v-model="colorModel"
            :label="t('tags.dialog.colorLabel')"
            outlined
            maxlength="7"
            :hint="t('tags.dialog.colorFormatHint')"
            :error="colorRuleState.error"
            :error-message="colorRuleState.message"
            :disable="isSaving"
            @update:model-value="onColorInput"
          >
            <template #append>
              <q-btn
                flat
                round
                dense
                icon="palette"
                :aria-label="t('tags.dialog.colorLabel')"
                :disable="isSaving"
              >
                <q-popup-proxy cover transition-show="scale" transition-hide="scale">
                  <q-color
                    :model-value="colorPickerValue"
                    format-model="hex"
                    no-header-tabs
                    no-footer
                    default-view="palette"
                    @update:model-value="onColorPicked"
                  />
                </q-popup-proxy>
              </q-btn>
            </template>
          </q-input>

          <div class="text-caption" style="color: var(--text-muted)">
            {{ t('tags.dialog.colorHint') }}
          </div>
        </q-card-section>

        <q-card-actions align="right" class="text-primary q-px-md q-pb-md">
          <q-btn
            flat
            :label="t('common.cancel')"
            :disable="isSaving"
            @click="dialogModel = false"
          />
          <q-btn
            color="primary"
            :label="t('common.save')"
            type="submit"
            :loading="isSaving"
            :disable="isLoadingTag"
          />
        </q-card-actions>
      </q-form>
    </q-card>
  </q-dialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUI } from 'src/composables/useUI';
import { TagsService } from 'src/services/tags';
import type { Rule, RuleErrorState } from 'src/types';
import type { CreateTagDto, UpdateTagDto } from 'src/types/tag';
import { executeRules } from 'src/utils/executeRules';
import { getTagTextContrastColor } from 'src/utils/tagColorContrast';

const HEX_COLOR_REGEX = /^#(?:[0-9A-F]{3}|[0-9A-F]{6})$/i;
const DEFAULT_TAG_COLOR = '#5865F2';

interface Props {
  modelValue: boolean;
  tagId: string | null;
}

const props = defineProps<Props>();
const { t } = useI18n();
const { notify, notifyError } = useUI();

const getNameRules = (): Rule<string>[] => [
  (value: string) => !value.trim() && t('tags.dialog.validation.nameRequired'),
  (value: string) => value.trim().length > 50 && t('tags.dialog.validation.nameLength'),
];

const getColorRules = (): Rule<string>[] => [
  (value: string) => !value.trim() && t('tags.dialog.validation.colorRequired'),
  (value: string) => !HEX_COLOR_REGEX.test(value.trim()) && t('tags.dialog.validation.colorFormat'),
];

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'saved'): void;
}>();

const form = reactive<CreateTagDto>({
  name: '',
  color: DEFAULT_TAG_COLOR,
});

const isSaving = ref(false);
const isLoadingTag = ref(false);
const nameRuleState = ref<RuleErrorState>({ error: false, message: '' });
const colorRuleState = ref<RuleErrorState>({ error: false, message: '' });

const editingId = computed(() => props.tagId);

const resetRuleState = (state: { value: RuleErrorState }) => {
  state.value.error = false;
  state.value.message = '';
};

const resetValidation = () => {
  resetRuleState(nameRuleState);
  resetRuleState(colorRuleState);
};

const resetForm = () => {
  form.name = '';
  form.color = DEFAULT_TAG_COLOR;
};

const normalizeColorInput = (value: string): string => {
  const upperValue = value.toUpperCase();
  const sanitized = upperValue.replace(/[^0-9A-F#]/g, '');
  const hasHashPrefix = sanitized.startsWith('#');
  const hexBody = sanitized.replace(/#/g, '').slice(0, 6);

  return `${hasHashPrefix || hexBody ? '#' : ''}${hexBody}`;
};

const previewColor = computed(() => {
  if (!HEX_COLOR_REGEX.test(form.color)) {
    return DEFAULT_TAG_COLOR;
  }

  return form.color.toUpperCase();
});

const previewStyle = computed(() => ({
  backgroundColor: previewColor.value,
  color: getTagTextContrastColor(previewColor.value),
  border: '1px solid var(--border-color)',
}));

const previewName = computed(() => {
  const name = form.name.trim();
  return name.length > 0 ? name : t('tags.dialog.previewNameFallback');
});

const colorPickerValue = computed(() => previewColor.value);

const dialogModel = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit('update:modelValue', value),
});

const nameModel = computed({
  get: () => form.name,
  set: (value: string) => {
    form.name = value;
  },
});

const colorModel = computed({
  get: () => form.color,
  set: (value: string) => {
    form.color = normalizeColorInput(value);
  },
});

const onNameInput = () => {
  if (nameRuleState.value.error) {
    resetRuleState(nameRuleState);
  }
};

const onColorInput = (value: string | number | null) => {
  const rawValue = typeof value === 'string' ? value : String(value ?? '');
  form.color = normalizeColorInput(rawValue);

  if (colorRuleState.value.error) {
    resetRuleState(colorRuleState);
  }
};

const onColorPicked = (value: string | null) => {
  form.color = normalizeColorInput(value ?? '');

  if (colorRuleState.value.error) {
    resetRuleState(colorRuleState);
  }
};

const validateForm = (): boolean => {
  resetValidation();
  const hasNameError = executeRules(nameRuleState, getNameRules(), form.name, t);
  const hasColorError = executeRules(colorRuleState, getColorRules(), form.color, t);
  return hasNameError || hasColorError;
};

const loadTagForEdit = async (tagId: string) => {
  isLoadingTag.value = true;
  try {
    const tag = await TagsService.getById(tagId);
    form.name = tag.name;
    form.color = normalizeColorInput(tag.color);
  } catch (err: unknown) {
    notifyError(t('tags.messages.getError'), err);
    dialogModel.value = false;
  } finally {
    isLoadingTag.value = false;
  }
};

const onSubmit = async () => {
  if (isLoadingTag.value || isSaving.value) return;
  if (validateForm()) return;

  const createPayload: CreateTagDto = {
    name: form.name.trim(),
    color: form.color.trim().toUpperCase(),
  };

  isSaving.value = true;
  try {
    if (editingId.value) {
      const updatePayload: UpdateTagDto = {
        name: createPayload.name,
        color: createPayload.color,
      };
      await TagsService.update(editingId.value, updatePayload);
      notify(t('tags.messages.updateSuccess'), 'positive');
    } else {
      await TagsService.create(createPayload);
      notify(t('tags.messages.createSuccess'), 'positive');
    }

    dialogModel.value = false;
    emit('saved');
  } catch (err: unknown) {
    notifyError(t('tags.messages.saveError'), err);
  } finally {
    isSaving.value = false;
  }
};

const onDialogClosed = () => {
  resetValidation();
  resetForm();
};

watch(
  () => [props.modelValue, props.tagId] as const,
  ([isOpen, tagId]) => {
    if (!isOpen) return;

    resetValidation();
    resetForm();

    if (tagId) {
      void loadTagForEdit(tagId);
    }
  },
  { immediate: true },
);
</script>
