<template>
  <div v-if="items.length > 0" class="markdown-attachments q-pa-sm">
    <div class="row items-center justify-between q-mb-xs">
      <div class="text-caption text-weight-medium">{{ t('attachments.form.sectionTitle') }}</div>
      <div class="text-caption" :class="exceeded ? 'text-negative' : ''">
        {{
          t('attachments.form.totalSize', {
            used: usedLabel,
            max: maxLabel,
          })
        }}
      </div>
    </div>
    <AttachmentItemsList
      :items="items"
      :empty-text="t('attachments.list.empty')"
      item-action-icon="close"
      :item-action-aria-label="t('attachments.form.removeFile')"
      @item-action="(id) => emit('remove', id)"
    />
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import AttachmentItemsList from 'src/components/AttachmentItemsList.vue';
import type { AttachmentListItem } from 'src/types/attachment';

interface Props {
  items: AttachmentListItem[];
  usedLabel: string;
  maxLabel: string;
  exceeded: boolean;
}

defineProps<Props>();

const emit = defineEmits<{
  (e: 'remove', id: string): void;
}>();

const { t } = useI18n();
</script>
