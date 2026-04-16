<template>
  <div class="attachment-items-list">
    <div class="relative-position">
      <q-list v-if="items.length > 0" class="q-gutter-y-xs">
        <q-item
          v-for="item in items"
          :key="item.id"
          :clickable="clickable"
          class="attachment-item q-px-sm q-py-xs"
          :class="{ 'attachment-item--clickable': clickable }"
          @click="onItemClick(item.id)"
        >
          <q-item-section avatar>
            <q-icon :name="item.icon" style="color: var(--text-muted)" />
          </q-item-section>

          <q-item-section>
            <q-item-label lines="1">{{ item.name }}</q-item-label>
            <q-item-label caption>{{ item.sizeLabel }}</q-item-label>
          </q-item-section>

          <q-item-section v-if="itemActionIcon" side>
            <q-btn
              flat
              round
              dense
              :icon="itemActionIcon"
              :aria-label="itemActionAriaLabel"
              @click.stop="emit('item-action', item.id)"
            />
          </q-item-section>
        </q-item>
      </q-list>

      <div v-else class="text-caption q-py-md" style="color: var(--text-muted)">
        {{ emptyText }}
      </div>

      <q-inner-loading :showing="loading" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AttachmentListItem } from 'src/types/attachment';

interface Props {
  items: AttachmentListItem[];
  emptyText: string;
  loading?: boolean;
  clickable?: boolean;
  itemActionIcon?: string;
  itemActionAriaLabel?: string;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  clickable: false,
  itemActionIcon: '',
  itemActionAriaLabel: '',
});

const emit = defineEmits<{
  (e: 'item-click', itemId: string): void;
  (e: 'item-action', itemId: string): void;
}>();

const onItemClick = (itemId: string) => {
  if (!props.clickable) return;
  emit('item-click', itemId);
};
</script>

<style scoped>
.attachment-item {
  border-radius: 8px;
}

.attachment-item--clickable:hover {
  background-color: var(--bg-hover);
}
</style>
