<template>
  <q-card flat bordered>
    <q-card-section>
      <div class="text-h6">{{ $t('settings.app.title') }}</div>
      <div class="text-caption" style="color: var(--text-muted)">
        {{ $t('settings.app.subtitle') }}
      </div>
    </q-card-section>

    <q-separator />

    <q-card-section class="row">
      <q-item
        v-for="item in items"
        :key="item.path"
        clickable
        class="col-12 col-md-3 column items-center justify-center"
        :to="`/${item.path}`"
      >
        <q-icon size="lg" :name="item.icon" />
        <span class="text-subtitle2">{{ $t(`settings.app.${item.label}`) }}</span>
      </q-item>
    </q-card-section>
  </q-card>
</template>
<script setup lang="ts">
const items = [
  { icon: 'sell', path: 'tags', label: 'tags' },
  { icon: 'view_kanban', path: 'task-statuses', label: 'taskStatuses' },
  { icon: 'extension', path: 'categories', label: 'categories' },
  { icon: 'list_alt', path: 'activities', label: 'activities' },
];
</script>
