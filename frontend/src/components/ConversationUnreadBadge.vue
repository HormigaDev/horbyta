<template>
  <q-badge v-if="count" color="negative" :aria-label="t('scope.unreadMessages', { count })">{{
    count > 99 ? '99+' : count
  }}</q-badge>
</template>
<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConversationStore } from 'src/stores/conversation';
const props = defineProps<{ projectId: string }>();
const { t } = useI18n();
const store = useConversationStore();
const count = computed(() => store.unread(props.projectId));
</script>
