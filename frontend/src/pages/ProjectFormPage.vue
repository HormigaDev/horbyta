<template>
  <q-page class="q-pa-md">
    <div>
      <q-card class="q-mb-md" flat bordered>
        <q-card-section>
          <div class="text-h6">
            {{ isNew ? t('projects.form.createTitle') : t('projects.form.editTitle') }}
          </div>
          <div class="text-caption text-grey" v-if="!isNew">
            {{ t('projects.form.editingPrefix', { id: project.id }) }}
          </div>
        </q-card-section>

        <q-card-section class="q-pt-none row q-col-gutter-md">
          <div class="col-12 col-md-4">
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
          <div class="row q-col-gutter-md" style="min-height: 50vh">
            <div class="col-12 col-md-6" style="display: flex; flex-direction: column">
              <MarkdownRenderer
                v-model="projectDescription"
                :edit-mode="true"
                class="full-height fit-textarea"
                @update:model-value="(val) => validateDescription(val ? val : null)"
              />
              <div v-if="descriptionRuleState.error" class="text-negative text-caption q-mt-xs">
                {{ descriptionRuleState.message }}
              </div>
            </div>
            <div class="col-12 col-md-6" style="display: flex; flex-direction: column">
              <div
                class="bordered rounded-borders q-pa-md full-height"
                style="
                  border: 1px solid var(--border-color);
                  background-color: var(--card-bg);
                  overflow-y: auto;
                  flex-grow: 1;
                "
              >
                <div class="text-caption text-grey q-mb-sm">
                  {{ t('projects.form.markdownPreview') }}
                </div>
                <MarkdownRenderer
                  :model-value="project.description || t('projects.form.emptyDescription')"
                />
              </div>
            </div>
          </div>

          <div v-if="isNew" class="q-mt-lg">
            <q-separator spaced />

            <div class="row items-center justify-between q-mb-sm">
              <div class="text-subtitle2">{{ t('attachments.form.sectionTitle') }}</div>
              <div
                class="text-caption"
                :class="filesBytesExceeded ? 'text-negative' : 'text-grey-7'"
              >
                {{
                  t('attachments.form.totalSize', {
                    used: formatStorageBytes(selectedFilesTotalBytes),
                    max: formatStorageBytes(MAX_ATTACHMENT_TOTAL_BYTES),
                  })
                }}
              </div>
            </div>

            <q-file
              :model-value="selectedFiles"
              multiple
              clearable
              dense
              outlined
              use-chips
              counter
              :label="t('attachments.form.inputLabel')"
              :hint="t('attachments.form.inputHint')"
              @update:model-value="onFilesSelected"
            />

            <AttachmentItemsList
              v-if="selectedFileItems.length > 0"
              class="q-mt-sm"
              :items="selectedFileItems"
              :empty-text="t('attachments.list.empty')"
              item-action-icon="close"
              :item-action-aria-label="t('attachments.form.removeFile')"
              @item-action="removeFileById"
            />
          </div>
        </q-card-section>
      </q-card>

      <div class="row justify-end q-gutter-sm">
        <q-btn :label="t('common.cancel')" flat color="grey" @click="goBack" />
        <q-btn
          :label="t('projects.form.saveButton')"
          color="primary"
          unelevated
          :loading="isSubmitting"
          @click.stop="saveProject"
        />
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import AttachmentItemsList from 'src/components/AttachmentItemsList.vue';
import NumericTextInput from 'src/components/NumericTextInput.vue';
import { ProjectsService } from 'src/services/projects';
import { CategoriesService } from 'src/services/categories';
import { useUI } from 'src/composables/useUI';
import { useProjectsRules } from 'src/composables/rules/projects.rules';
import { useSelectByTerm } from 'src/composables/useSelectByTerm';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import SelectByTerm from 'src/components/SelectByTerm.vue';
import type { AttachmentListItem } from 'src/types/attachment';
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
import {
  MAX_ATTACHMENT_FILE_BYTES,
  MAX_ATTACHMENT_TOTAL_BYTES,
  formatStorageBytes,
  resolveFileIcon,
  sumFileSizes,
  toFileArray,
  type QFileModelValue,
} from 'src/utils/attachments';
import { normalizeNumericText, parseNumericText } from 'src/utils/numericInput';
import { CURRENCY_TYPE_OPTIONS } from 'src/data/constants';

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
const selectedFiles = ref<File[]>([]);
const selectedFilesTotalBytes = computed(() => sumFileSizes(selectedFiles.value));
const filesBytesExceeded = computed(
  () => selectedFilesTotalBytes.value > MAX_ATTACHMENT_TOTAL_BYTES,
);

const billingForm = ref<ProjectBillingFormState>({
  billing_type: 'fixed',
  currency_type: 'USD',
  hourly_rate: '',
  fixed_price: '',
  retainer_amount: '',
  payment_day: '',
});

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

const buildSelectedFileId = (file: File, index: number): string => {
  return `${file.name}-${file.lastModified}-${index}`;
};

const selectedFileItems = computed<AttachmentListItem[]>(() => {
  return selectedFiles.value.map((file, index) => ({
    id: buildSelectedFileId(file, index),
    name: file.name,
    sizeLabel: formatStorageBytes(file.size),
    icon: resolveFileIcon(file),
  }));
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

const project = ref<ProjectFormStateDto>({
  id: '',
  name: '',
  description: '',
  status: 'open',
  billable: true,
  category_id: null,
  client_id: null,
});
const projectDescription = computed({
  get: () => project.value.description || '',
  set: (val: string) => {
    project.value.description = val ? val : null;
  },
});

const onFilesSelected = (modelValue: QFileModelValue) => {
  const incomingFiles = toFileArray(modelValue);

  const oversizedFile = incomingFiles.find((file) => file.size > MAX_ATTACHMENT_FILE_BYTES);
  if (oversizedFile) {
    notify(
      t('attachments.form.singleFileLimitExceeded', {
        file: oversizedFile.name,
        max: formatStorageBytes(MAX_ATTACHMENT_FILE_BYTES),
      }),
      'warning',
    );
    return;
  }

  const nextTotal = sumFileSizes(incomingFiles);
  if (nextTotal > MAX_ATTACHMENT_TOTAL_BYTES) {
    notify(
      t('attachments.form.totalLimitExceeded', {
        max: formatStorageBytes(MAX_ATTACHMENT_TOTAL_BYTES),
      }),
      'warning',
    );
    return;
  }

  selectedFiles.value = incomingFiles;
};

const removeFileById = (itemId: string) => {
  selectedFiles.value = selectedFiles.value.filter((file, index) => {
    return buildSelectedFileId(file, index) !== itemId;
  });
};

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
  } catch (err: unknown) {
    notifyError(t('projects.form.loadError'), err);
    void router.push('/projects');
  } finally {
    hideLoading();
  }
};

const goBack = () => {
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
      status: project.value.status,
      category_id: project.value.category_id || null,
      client_id: project.value.client_id || null,
      billable: project.value.billable,
      project_billing: projectBillingPayload,
    };

    if (isNew.value) {
      const created = await ProjectsService.create(payload, selectedFiles.value);
      notify(t('projects.form.createSuccess'), 'positive');
      void router.push(`/projects/${created.id}`);
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
      await ProjectsService.update(project.value.id, updatePayload);
      notify(t('projects.form.updateSuccess'), 'positive');
      void router.push(`/projects/${project.value.id}`);
    }
  } catch (err: unknown) {
    notifyError(t('projects.form.saveError'), err);
  } finally {
    isSubmitting.value = false;
    hideLoading();
  }
};

onMounted(() => {
  void loadCategoriesByTerm('');
  void loadClientsByTerm('');
  void loadProject();
});
</script>

<style scoped>
.fit-textarea :deep(.q-field__control),
.fit-textarea :deep(.q-field__native) {
  height: 100%;
  min-height: 50vh;
}

.text-grey-7 {
  color: var(--text-muted) !important;
}
</style>
