<template>
  <dl id="project-information" tabindex="-1" class="project-facts project-section-anchor">
    <slot />
    <div>
      <dt>{{ t('projects.detail.billingFact') }}</dt>
      <dd v-if="project.billable && billing">
        {{ billingAmount }}
        <span class="project-facts__caption">{{ billingCaption }}</span>
      </dd>
      <dd v-else-if="!project.billable">{{ t('projects.detail.noCost') }}</dd>
      <dd v-else class="project-facts__caption">{{ t('projects.detail.billingEmpty') }}</dd>
    </div>
    <div>
      <dt>{{ t('common.created') }}</dt>
      <dd>{{ date(project.created_at) }}</dd>
    </div>
    <div>
      <dt>{{ t('common.updated') }}</dt>
      <dd>{{ date(project.updated_at) }}</dd>
    </div>
  </dl>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useNumberFormatter } from 'src/composables/useNumberFormatter';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import { formatDateByPreferences } from 'src/utils/dateFormatting';
import type { ProjectDetailDto } from 'src/types/project';

const props = defineProps<{
  project: Pick<ProjectDetailDto, 'billable' | 'project_billing' | 'created_at' | 'updated_at'>;
}>();
const { t } = useI18n();
const { formatCurrency } = useNumberFormatter();
const preferences = useDisplayPreferences();

const billing = computed(() => props.project.project_billing);
const date = (value: string) =>
  formatDateByPreferences(value, {
    timezone: preferences.timezone,
    dateFormat: preferences.dateFormat,
    fallback: t('common.empty'),
  });
const billingAmount = computed(() => {
  const value = billing.value;
  if (!value) return '';
  const amount = {
    fixed: value.fixed_price,
    hourly: value.hourly_rate,
    retainer: value.retainer_amount,
  }[value.billing_type];
  return amount ? formatCurrency(amount, value.currency_type) : value.currency_type;
});
const billingCaption = computed(() => {
  const value = billing.value;
  if (!value) return '';
  if (value.billing_type === 'retainer' && value.payment_day != null) {
    return t('projects.detail.retainerCaption', { day: value.payment_day });
  }
  return t(`projects.billing.type.${value.billing_type}`);
});
</script>
