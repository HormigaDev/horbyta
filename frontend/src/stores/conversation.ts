import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { ScopeService } from 'src/services/scope';
import type { ClientNotice } from 'src/types/scope';
export interface ConversationNotice {
  project_id: string;
  project_name: string;
  unread_count: number;
  message_id: string;
  recorded_at: string;
  preview: string | null;
}
export const useConversationStore = defineStore('conversation', () => {
  const notices = ref<ConversationNotice[]>([]);
  // Portal del cliente: acciones pendientes y su historial (persistidos en el servidor).
  const clientNotices = ref<ClientNotice[]>([]);
  const pendingActions = computed(() => clientNotices.value.filter((n) => !n.resolved));
  const client = ref(false);
  let generation = 0;
  let request = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;
  const total = computed(() => notices.value.reduce((sum, n) => sum + n.unread_count, 0));
  const unread = (project: string) =>
    notices.value.find((n) => n.project_id === project)?.unread_count ?? 0;
  function configure(portal: boolean) {
    generation++;
    client.value = portal;
    notices.value = [];
    clientNotices.value = [];
    if (timer) clearTimeout(timer);
    void refresh();
  }
  async function refresh() {
    const current = generation;
    const sequence = ++request;
    try {
      const [result, actions] = await Promise.all([
        ScopeService.conversationInbox(client.value),
        client.value ? ScopeService.notices() : Promise.resolve(null),
      ]);
      if (current === generation && sequence === request) {
        notices.value = result;
        if (actions) clientNotices.value = actions;
      }
    } catch {
      /* Reconnection retries; do not discard unread state on network failure. */
    }
  }
  function scheduleRefresh() {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      void refresh();
    }, 120);
  }
  async function read(project: string, message: string) {
    await ScopeService.readConversation(project, message, client.value);
    await refresh();
  }
  return {
    notices,
    clientNotices,
    pendingActions,
    total,
    unread,
    configure,
    refresh,
    scheduleRefresh,
    read,
  };
});
