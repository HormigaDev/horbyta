<template>
  <q-expansion-item
    dense
    switch-toggle-side
    expand-separator
    class="error-node"
    :header-class="'error-node__header'"
  >
    <template #header>
      <q-item-section>
        <q-item-label class="error-node__label">{{ label }}</q-item-label>
      </q-item-section>
      <q-item-section side class="error-node__preview">
        <span :class="`error-node__value error-node__value--${kind}`">{{ preview }}</span>
      </q-item-section>
    </template>

    <div class="error-node__body">
      <template v-if="children">
        <ErrorDetailsNode
          v-for="child in children"
          :key="child.key"
          :name="child.key"
          :value="child.value"
          :depth="depth + 1"
        />
        <div v-if="children.length === 0" class="error-node__empty">
          {{ t('error.details.empty') }}
        </div>
      </template>
      <div v-else class="error-node__box">
        <pre :class="`error-node__text error-node__value--${kind}`">{{ text }}</pre>
        <q-btn
          flat
          round
          dense
          size="sm"
          icon="content_copy"
          class="error-node__copy"
          :aria-label="t('error.details.copy')"
          @click="copy(text)"
        />
      </div>
    </div>
  </q-expansion-item>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { copyToClipboard, useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';

defineOptions({ name: 'ErrorDetailsNode' });

const props = withDefaults(defineProps<{ name: string; value: unknown; depth?: number }>(), {
  depth: 0,
});

const i18n = useI18n();
const { t } = i18n;
const $q = useQuasar();

// Nombres legibles para las claves conocidas del error; el resto se muestra tal cual.
const label = computed(() => {
  const key = `error.details.key.${props.name}`;
  const known = Object.prototype.hasOwnProperty.call(i18n.getLocaleMessage(i18n.locale.value), key);
  return props.depth === 0 && known ? t(key) : props.name;
});

const kind = computed(() => {
  const value = props.value;
  if (value === null || value === undefined) return 'null';
  if (Array.isArray(value)) return 'array';
  return typeof value === 'object' ? 'object' : typeof value;
});

const children = computed(() => {
  const value = props.value;
  if (Array.isArray(value)) return value.map((item, index) => ({ key: `[${index}]`, value: item }));
  if (value && typeof value === 'object') {
    return Object.entries(value as Record<string, unknown>).map(([key, item]) => ({
      key,
      value: item,
    }));
  }
  return null;
});

const text = computed(() => {
  const value = props.value;
  if (value === null || value === undefined) return 'null';
  return typeof value === 'string' ? value : JSON.stringify(value);
});

const preview = computed(() => {
  if (kind.value === 'array') return t('error.details.items', (props.value as unknown[]).length);
  if (kind.value === 'object') return t('error.details.fields', children.value?.length ?? 0);
  return text.value.length > 60 ? `${text.value.slice(0, 60)}…` : text.value;
});

const copy = async (value: string) => {
  try {
    await copyToClipboard(value);
    $q.notify({
      message: t('error.details.copied'),
      type: 'positive',
      position: 'top-right',
      timeout: 1500,
    });
  } catch {
    $q.notify({ message: t('error.details.copyFailed'), type: 'warning', position: 'top-right' });
  }
};
</script>

<style scoped>
.error-node :deep(.error-node__header) {
  min-height: 40px;
  padding: 4px 8px;
}

.error-node__label {
  font-weight: 600;
  color: var(--text-color);
}

.error-node__preview {
  flex: 0 1 auto;
  min-width: 0;
  max-width: 55%;
  overflow: hidden;
}

.error-node :deep(.q-item__section--main) {
  flex: 1 0 auto;
}

.error-node__value {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: block;
}

.error-node__value--string {
  color: var(--success-text);
}
.error-node__value--number {
  color: var(--info-text);
}
.error-node__value--boolean,
.error-node__value--null {
  color: var(--warning-text);
}
.error-node__value--object,
.error-node__value--array {
  color: var(--text-muted);
}

.error-node__body {
  padding: 4px 8px 8px 28px;
}

.error-node__box {
  position: relative;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--surface-page);
}

.error-node__text {
  margin: 0;
  padding: 10px 40px 10px 12px;
  max-height: 220px;
  overflow: auto;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12.5px;
  line-height: 1.5;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.error-node__copy {
  position: absolute;
  top: 4px;
  right: 4px;
  color: var(--text-muted);
}

.error-node__empty {
  color: var(--text-muted);
  font-size: 12px;
  padding: 4px 0;
}
</style>
