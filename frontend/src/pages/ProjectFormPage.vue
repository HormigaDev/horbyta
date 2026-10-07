<template>
  <q-page class="form-page q-pa-md">
    <div class="padding-page">
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6 flex items-center">
            <q-icon v-if="isNew" name="create_new_folder" size="md" class="q-mr-md" />
            <q-img
              src="/icons/bookmark_manager.svg"
              v-else
              width="32px"
              height="32px"
              class="q-mr-md"
            />
            {{ isNew ? t('projects.form.createTitle') : t('projects.form.editTitle') }}
          </div>
          <div class="text-caption text-grey" v-if="!isNew">
            {{ t('projects.form.editingPrefix', { id: project.id }) }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 form-section-title">{{ t('ui.identification') }}</div>
          <div class="col-12">
            <q-input
              v-model="project.name"
              :label="t('projects.form.nameLabel')"
              outlined
              dense
              :error="nameRuleState.error"
              :error-message="nameRuleState.message"
              @update:model-value="validateName(project.name)"
            />
          </div>
          <div class="col-12 col-md-4">
            <SelectByTerm
              v-model="project.category_id"
              :label="t('projects.form.categoryLabel')"
              :options="categoryOptions"
              :loading="isCategoriesLoading"
              @search="onCategorySearch"
            />
          </div>
          <div class="col-12 col-md-4">
            <SelectByTerm
              v-model="project.client_id"
              :label="$t('projects.form.clientLabel')"
              :options="clientOptions"
              :loading="isClientsLoading"
              @search="onClientSearch"
            />
          </div>
          <div class="col-12 col-md-4" v-if="!isNew">
            <q-select
              v-model="project.status"
              :label="$t('projects.form.statusLabel')"
              :options="getProjectStatusOptions($t)"
              emit-value
              map-options
              outlined
              dense
            />
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none">
          <q-separator spaced />

          <div class="text-subtitle1 q-mb-sm">{{ t('projects.form.billingTitle') }}</div>

          <div class="row q-col-gutter-md">
            <div class="col-12">
              <q-toggle
                v-model="project.billable"
                :label="t('projects.form.billableLabel')"
                color="primary"
              />
              <div class="text-caption" style="color: var(--text-muted)">
                {{ t('projects.form.billableHint') }}
              </div>
            </div>

            <div class="col-12 col-md-4" v-if="project.billable">
              <q-select
                v-model="billingForm.currency_type"
                :label="t('projects.form.currencyTypeLabel')"
                :options="CURRENCY_TYPE_OPTIONS"
                outlined
                dense
                emit-value
                map-options
                :error="hasCurrencyTypeError"
                :error-message="t('projects.form.billingValidation.currencyType')"
                @update:model-value="onCurrencyTypeInput"
              />
            </div>

            <div class="col-12 col-md-4" v-if="project.billable">
              <q-select
                v-model="billingForm.billing_type"
                :label="t('projects.form.billingTypeLabel')"
                :options="billingTypeOptions"
                emit-value
                map-options
                outlined
                dense
              />
            </div>

            <div
              class="col-12 col-md-4"
              v-if="project.billable && billingForm.billing_type === 'fixed'"
            >
              <NumericTextInput
                v-model="billingForm.fixed_price"
                :label="t('projects.form.fixedPriceLabel')"
                :decimal-places="4"
                :error="hasFixedPriceError"
              />
              <div v-if="hasFixedPriceError" class="text-negative text-caption q-mt-xs">
                {{ t('projects.form.billingValidation.fixedPrice') }}
              </div>
            </div>

            <div
              class="col-12 col-md-4"
              v-if="project.billable && billingForm.billing_type === 'hourly'"
            >
              <NumericTextInput
                v-model="billingForm.hourly_rate"
                :label="t('projects.form.hourlyRateLabel')"
                :decimal-places="4"
                :error="hasHourlyRateError"
              />
              <div v-if="hasHourlyRateError" class="text-negative text-caption q-mt-xs">
                {{ t('projects.form.billingValidation.hourlyRate') }}
              </div>
            </div>

            <template v-if="project.billable && billingForm.billing_type === 'retainer'">
              <div class="col-12 col-md-4">
                <NumericTextInput
                  v-model="billingForm.retainer_amount"
                  :label="t('projects.form.retainerAmountLabel')"
                  :decimal-places="4"
                  :error="hasRetainerAmountError"
                />
                <div v-if="hasRetainerAmountError" class="text-negative text-caption q-mt-xs">
                  {{ t('projects.form.billingValidation.retainerAmount') }}
                </div>
              </div>

              <div class="col-12 col-md-4">
                <NumericTextInput
                  v-model="billingForm.payment_day"
                  :label="t('projects.form.paymentDayLabel')"
                  :only-integers="true"
                  :error="hasPaymentDayError"
                />
                <div v-if="hasPaymentDayError" class="text-negative text-caption q-mt-xs">
                  {{ t('projects.form.billingValidation.paymentDay') }}
                </div>
              </div>
            </template>
          </div>
        </q-card-section>
      </q-card>

      <q-card flat bordered class="q-mb-md">
        <q-card-section>
          <div class="text-subtitle1 q-mb-sm">{{ t('projects.form.descriptionTitle') }}</div>
          <MarkdownRenderer
            ref="markdownRef"
            v-model="projectDescription"
            v-model:pending-files="markdownFiles"
            edit-mode
            enable-preview-toggle
            enable-attachments
            @update:model-value="(val) => validateDescription(val ? val : null)"
          />
          <div v-if="descriptionRuleState.error" class="text-negative text-caption q-mt-xs">
            {{ descriptionRuleState.message }}
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm form-actions">
        <q-btn
          v-if="isNew"
          flat
          round
          icon="bookmarks"
          :aria-label="t('formPreferences.title')"
          color="grey"
          @click="prefsDialogOpen = true"
        >
          <q-tooltip>{{ t('formPreferences.title') }}</q-tooltip>
        </q-btn>
        <q-btn
          flat
          round
          icon="push_pin"
          :aria-label="t('common.keepFormOpen')"
          :color="pinButtonColor"
          :aria-pressed="stayOnFormAfterSave"
          @click="stayOnFormAfterSave = !stayOnFormAfterSave"
        >
          <q-tooltip>{{ t('common.keepFormOpen') }}</q-tooltip>
        </q-btn>
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          :label="t('projects.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveProject"
        />
      </div>

      <FormPreferencesDialog
        v-model="prefsDialogOpen"
        form-key="project"
        :current-fields="currentPreferenceFields"
        @apply="applyPreference"
      />
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import FormPreferencesDialog from 'src/components/FormPreferencesDialog.vue';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { ProjectsService } from 'src/services/projects';
import { CategoriesService } from 'src/services/categories';
import { useFormDraft } from 'src/composables/useFormDraft';
import { useFormPreferences } from 'src/composables/useFormPreferences';
import { usePinnedFormSave } from 'src/composables/usePinnedFormSave';
import { useUI } from 'src/composables/useUI';
import { useUnsavedChanges } from 'src/composables/useUnsavedChanges';
import { useProjectsRules } from 'src/composables/rules/projects.rules';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import type { MarkdownPendingFile } from 'src/types/markdown';
import type {
  CreateProjectDto,
  ProjectBillingType,
  ProjectFormStateDto,
  UpdateProjectDto,
  UpsertProjectBillingDto,
} from 'src/types/project';
import type { CategorySummaryDto } from 'src/types/category';
import type { ClientSummaryDto } from 'src/types/clients';
import { ClientsService } from 'src/services/clients';
import { normalizeNumericText, parseNumericText } from 'src/utils/numericInput';
import { CURRENCY_TYPE_OPTIONS, getProjectStatusOptions } from 'src/data/constants';

interface ProjectBillingFormState {
  billing_type: ProjectBillingType;
  currency_type: string;
  hourly_rate: string;
  fixed_price: string;
  retainer_amount: string;
  payment_day: string;
}

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const { notify, notifyError, showLoading, hideLoading } = useUI();
const { nameRuleState, descriptionRuleState, validateName, validateDescription } =
  useProjectsRules();

const isNew = computed(() => route.name === 'project-create');
const isSubmitting = ref(false);
const hasAttemptedSubmit = ref(false);
const hasChanges = ref(false);
const { stayOnFormAfterSave, pinButtonColor, handlePinnedSaveSuccess } = usePinnedFormSave();
const markdownFiles = ref<MarkdownPendingFile[]>([]);
const markdownRef = ref<{ resetToEdit: () => void } | null>(null);

watch(stayOnFormAfterSave, () => {
  markdownRef.value?.resetToEdit();
});

const createDefaultBillingForm = (): ProjectBillingFormState => ({
  billing_type: 'fixed',
  currency_type: 'USD',
  hourly_rate: '',
  fixed_price: '',
  retainer_amount: '',
  payment_day: '',
});

const billingForm = ref<ProjectBillingFormState>(createDefaultBillingForm());

const billingTypeOptions = computed(() => {
  return [
    {
      label: t('projects.billing.type.fixed'),
      value: 'fixed' as ProjectBillingType,
    },
    {
      label: t('projects.billing.type.hourly'),
      value: 'hourly' as ProjectBillingType,
    },
    {
      label: t('projects.billing.type.retainer'),
      value: 'retainer' as ProjectBillingType,
    },
  ];
});

const normalizedCurrencyType = computed(() => {
  return billingForm.value.currency_type.trim().toUpperCase();
});

const parsedFixedPrice = computed(() => {
  return parseNumericText(billingForm.value.fixed_price, { decimalPlaces: 4 });
});

const parsedHourlyRate = computed(() => {
  return parseNumericText(billingForm.value.hourly_rate, { decimalPlaces: 4 });
});

const parsedRetainerAmount = computed(() => {
  return parseNumericText(billingForm.value.retainer_amount, { decimalPlaces: 4 });
});

const parsedPaymentDay = computed(() => {
  return parseNumericText(billingForm.value.payment_day, { onlyIntegers: true });
});

const hasCurrencyTypeError = computed(() => {
  if (!hasAttemptedSubmit.value || !project.value.billable) return false;
  return normalizedCurrencyType.value.length !== 3;
});

const hasFixedPriceError = computed(() => {
  if (!hasAttemptedSubmit.value || !project.value.billable) return false;
  if (billingForm.value.billing_type !== 'fixed') return false;

  return Number.isNaN(parsedFixedPrice.value) || parsedFixedPrice.value < 0;
});

const hasHourlyRateError = computed(() => {
  if (!hasAttemptedSubmit.value || !project.value.billable) return false;
  if (billingForm.value.billing_type !== 'hourly') return false;

  return Number.isNaN(parsedHourlyRate.value) || parsedHourlyRate.value < 0;
});

const hasRetainerAmountError = computed(() => {
  if (!hasAttemptedSubmit.value || !project.value.billable) return false;
  if (billingForm.value.billing_type !== 'retainer') return false;

  return Number.isNaN(parsedRetainerAmount.value) || parsedRetainerAmount.value < 0;
});

const hasPaymentDayError = computed(() => {
  if (!hasAttemptedSubmit.value || !project.value.billable) return false;
  if (billingForm.value.billing_type !== 'retainer') return false;

  if (Number.isNaN(parsedPaymentDay.value)) return true;

  return parsedPaymentDay.value < 1 || parsedPaymentDay.value > 28;
});

const {
  options: categoryOptions,
  isLoading: isCategoriesLoading,
  load: loadCategoriesByTerm,
} = useSelectByTerm<CategorySummaryDto>(CategoriesService.search, {
  termColumn: 'name',
  orderBy: 'created_at',
  orderAsc: false,
  toOption: (category) => ({
    label: category.name,
    value: category.id,
  }),
});
const {
  options: clientOptions,
  isLoading: isClientsLoading,
  load: loadClientsByTerm,
} = useSelectByTerm<ClientSummaryDto>(ClientsService.search, {
  termColumn: 'name',
  orderBy: 'name',
  orderAsc: true,
  toOption: (client) => ({
    label: client.name,
    value: client.id,
  }),
});

const onCategorySearch = (term: string) => {
  if (term.trim().length === 0 && categoryOptions.value.length > 0) return;
  void loadCategoriesByTerm(term);
};
const onClientSearch = (term: string) => {
  if (term.trim().length === 0 && clientOptions.value.length > 0) return;
  loadClientsByTerm(term);
};

const createEmptyProjectForm = (): ProjectFormStateDto => ({
  id: '',
  name: '',
  description: '',
  status: 'open',
  billable: true,
  category_id: null,
  client_id: null,
});

const project = ref<ProjectFormStateDto>(createEmptyProjectForm());

const prefsDialogOpen = ref(false);
const { defaultPreference, load: loadPreferences } = useFormPreferences('project');

const currentPreferenceFields = computed(() => ({
  category_id: project.value.category_id,
  client_id: project.value.client_id,
  billable: project.value.billable,
}));

const isApplyingPreference = ref(false);

const applyPreference = (fields: Record<string, unknown>) => {
  isApplyingPreference.value = true;
  if (fields.category_id !== undefined)
    project.value.category_id = fields.category_id as string | null;
  if (fields.client_id !== undefined) project.value.client_id = fields.client_id as string | null;
  if (fields.billable !== undefined) project.value.billable = fields.billable as boolean;
  isApplyingPreference.value = false;
};

const billingBaseline = ref(JSON.stringify(billingForm.value));
const { clearDraft, restoreDraftIfAvailable } = useFormDraft(
  () => ({ ...project.value, _billingForm: { ...billingForm.value } }),
  (draft) => {
    const { _billingForm, ...projectDraft } = draft;
    Object.assign(project.value, projectDraft);
    if (_billingForm) billingForm.value = { ..._billingForm };
  },
  { formKey: 'project', enabled: () => !isSubmitting.value },
);

const isDirty = computed(() => {
  if (!hasChanges.value) return false;
  if (project.value.name.trim() !== '') return true;
  if (!defaultPreference.value) return true;
  const f = defaultPreference.value.fields;
  return !(
    project.value.category_id === ((f.category_id as string | null) ?? null) &&
    project.value.client_id === ((f.client_id as string | null) ?? null) &&
    project.value.billable === ((f.billable as boolean) ?? true)
  );
});

const { confirmLeave } = useUnsavedChanges(
  () =>
    !isSubmitting.value &&
    (isDirty.value ||
      markdownFiles.value.length > 0 ||
      JSON.stringify(billingForm.value) !== billingBaseline.value),
);

watch(
  project,
  () => {
    if (!isApplyingPreference.value) hasChanges.value = true;
  },
  { deep: true },
);

const resetProjectForm = async () => {
  markdownRef.value?.resetToEdit();
  hasAttemptedSubmit.value = false;
  nameRuleState.value = { error: false, message: '' };
  descriptionRuleState.value = { error: false, message: '' };

  hasChanges.value = false;

  if (isNew.value) {
    project.value = createEmptyProjectForm();
    billingForm.value = createDefaultBillingForm();
    markdownFiles.value = [];
    billingBaseline.value = JSON.stringify(billingForm.value);
    clearDraft();
    if (defaultPreference.value) {
      applyPreference(defaultPreference.value.fields);
    }
    return;
  }

  await loadProject();
};

const projectDescription = computed({
  get: () => project.value.description || '',
  set: (val: string) => {
    project.value.description = val ? val : null;
  },
});

const onCurrencyTypeInput = (value: string | number | null) => {
  billingForm.value.currency_type = String(value || '')
    .toUpperCase()
    .replace(/[^A-Z]/g, '')
    .slice(0, 3);
};

const normalizeBillingNumericValue = (value: string | null | undefined) => {
  if (!value) return '';

  return normalizeNumericText(String(value), {
    decimalPlaces: 4,
    keepIncompleteDecimals: false,
  });
};

const validateProjectBilling = (): boolean => {
  if (!project.value.billable) return true;

  if (normalizedCurrencyType.value.length !== 3) {
    return false;
  }

  if (billingForm.value.billing_type === 'fixed') {
    return !Number.isNaN(parsedFixedPrice.value) && parsedFixedPrice.value >= 0;
  }

  if (billingForm.value.billing_type === 'hourly') {
    return !Number.isNaN(parsedHourlyRate.value) && parsedHourlyRate.value >= 0;
  }

  if (billingForm.value.billing_type === 'retainer') {
    return (
      !Number.isNaN(parsedRetainerAmount.value) &&
      parsedRetainerAmount.value >= 0 &&
      !Number.isNaN(parsedPaymentDay.value) &&
      parsedPaymentDay.value >= 1 &&
      parsedPaymentDay.value <= 28
    );
  }

  return false;
};

const buildProjectBillingPayload = (): UpsertProjectBillingDto | null => {
  if (!project.value.billable) return null;

  const basePayload: UpsertProjectBillingDto = {
    currency_type: normalizedCurrencyType.value,
    billing_type: billingForm.value.billing_type,
  };

  if (billingForm.value.billing_type === 'fixed') {
    return {
      ...basePayload,
      fixed_price: String(parsedFixedPrice.value),
      hourly_rate: null,
      retainer_amount: null,
      payment_day: null,
    };
  }

  if (billingForm.value.billing_type === 'hourly') {
    return {
      ...basePayload,
      hourly_rate: String(parsedHourlyRate.value),
      fixed_price: null,
      retainer_amount: null,
      payment_day: null,
    };
  }

  return {
    ...basePayload,
    retainer_amount: String(parsedRetainerAmount.value),
    payment_day: parsedPaymentDay.value,
    fixed_price: null,
    hourly_rate: null,
  };
};

const loadProject = async () => {
  if (isNew.value) return;

  const projectId = route.params.id as string;
  showLoading(t('projects.form.loadingProject'));
  try {
    const data = await ProjectsService.getById(projectId);
    project.value = {
      id: data.id,
      name: data.name,
      description: data.description,
      status: data.status,
      billable: data.billable,
      category_id: data.category_id,
      client_id: data.client_id,
    };

    if (data.project_billing) {
      billingForm.value = {
        billing_type: data.project_billing.billing_type,
        currency_type: data.project_billing.currency_type.toUpperCase(),
        hourly_rate: normalizeBillingNumericValue(data.project_billing.hourly_rate),
        fixed_price: normalizeBillingNumericValue(data.project_billing.fixed_price),
        retainer_amount: normalizeBillingNumericValue(data.project_billing.retainer_amount),
        payment_day:
          data.project_billing.payment_day === null ||
          data.project_billing.payment_day === undefined
            ? ''
            : String(data.project_billing.payment_day),
      };
    } else {
      billingForm.value = createDefaultBillingForm();
    }

    if (data.category_id) {
      const exists = categoryOptions.value.some((option) => option.value === data.category_id);
      if (!exists) {
        try {
          const category = await CategoriesService.getById(data.category_id);
          categoryOptions.value = [
            { value: category.id, label: category.name },
            ...categoryOptions.value,
          ];
        } catch {
          // Ignorar: si la categoría fue eliminada, mantenemos el formulario operativo.
        }
      }
    }

    markdownFiles.value = [];
    billingBaseline.value = JSON.stringify(billingForm.value);
  } catch (err: unknown) {
    notifyError(t('projects.form.loadError'), err);
    void router.push('/projects');
  } finally {
    hideLoading();
  }
};

const goBack = async () => {
  if (!(await confirmLeave())) return;
  clearDraft();
  hasChanges.value = false;
  if (isNew.value) {
    void router.push('/projects');
  } else {
    void router.push(`/projects/${project.value.id}`);
  }
};

const saveProject = async () => {
  hasAttemptedSubmit.value = true;

  const hasNameError = validateName(project.value.name);
  const hasDescError = validateDescription(project.value.description || null);
  const hasBillingError = !validateProjectBilling();

  if (hasNameError || hasDescError || hasBillingError) return;

  const projectBillingPayload = buildProjectBillingPayload();

  isSubmitting.value = true;
  showLoading(t('projects.form.saving'));

  try {
    const payload: CreateProjectDto = {
      name: project.value.name,
      description: project.value.description || null,
      category_id: project.value.category_id || null,
      client_id: project.value.client_id || null,
      billable: project.value.billable,
      project_billing: projectBillingPayload,
    };

    if (isNew.value) {
      const created = await ProjectsService.create(payload, markdownFiles.value);
      notify(t('projects.form.createSuccess'), 'positive');
      hasChanges.value = false;
      clearDraft();
      await handlePinnedSaveSuccess({
        reset: resetProjectForm,
        redirect: () => router.push(`/projects/${created.id}`),
      });
    } else {
      const updatePayload: UpdateProjectDto = {
        name: payload.name,
        description: payload.description || null,
        status: project.value.status,
        category_id: payload.category_id || null,
        client_id: payload.client_id || null,
        billable: project.value.billable,
        project_billing: projectBillingPayload,
      };
      await ProjectsService.update(project.value.id, updatePayload, markdownFiles.value);
      notify(t('projects.form.updateSuccess'), 'positive');
      hasChanges.value = false;
      clearDraft();
      await handlePinnedSaveSuccess({
        reset: resetProjectForm,
        redirect: () => router.push(`/projects/${project.value.id}`),
      });
    }
  } catch (err: unknown) {
    notifyError(t('projects.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(async () => {
  void loadCategoriesByTerm('');
  void loadClientsByTerm('');
  if (isNew.value) {
    await loadPreferences();
    const restored = await restoreDraftIfAvailable();
    if (!restored && defaultPreference.value) {
      applyPreference(defaultPreference.value.fields);
    }
  }
  await loadProject();
  if (!isNew.value) await restoreDraftIfAvailable();
});
</script>

<style scoped>
.text-grey-7 {
  color: var(--text-muted) !important;
}
</style>
