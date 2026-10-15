import { onUnmounted, watch } from 'vue';
import { useQuasar } from 'quasar';
import { useI18n } from 'vue-i18n';
import { useRouter } from 'vue-router';
import { useConversationStore } from 'src/stores/conversation';
import { useAuthStore } from 'src/stores/auth';
import { useClientAuthStore } from 'src/stores/clientAuth';
import { useWebSocket } from './useWebSocket';

export interface ScopeConversationUpdate {
  project_id: string;
  change_request_id?: string | null;
  revision: number;
  event_id: string;
  kind: string;
  actor_kind: string;
  actor_name: string | null;
  project_name: string;
}
const listeners = new Set<(event: ScopeConversationUpdate | null) => void>();
const readers = new Set<(project: string, revision: number, change?: string) => boolean>();
export function onScopeConversationUpdate(
  handler: (event: ScopeConversationUpdate | null) => void,
) {
  listeners.add(handler);
  return () => {
    listeners.delete(handler);
  };
}
export function registerScopeConversationReader(
  reader: (project: string, revision: number, change?: string) => boolean,
) {
  readers.add(reader);
  return () => {
    readers.delete(reader);
  };
}
export function isScopeConversationFocused(project: string, revision: number, change?: string) {
  return (
    document.visibilityState === 'visible' &&
    document.hasFocus() &&
    [...readers].some((reader) => reader(project, revision, change))
  );
}
function emit(event: ScopeConversationUpdate | null) {
  for (const listener of listeners) listener(event);
}
export function useScopeRealtime(client = false, shared?: ReturnType<typeof useWebSocket>) {
  const websocket = shared ?? useWebSocket({ path: client ? '/ws/scope/client' : '/ws' });
  const inbox = useConversationStore();
  const auth = useAuthStore();
  const portalAuth = useClientAuthStore();
  watch(
    () => (client ? portalAuth.client?.id : auth.user?.id),
    () => inbox.configure(client),
    { immediate: true },
  );
  const $q = useQuasar();
  const { t } = useI18n();
  const router = useRouter();
  websocket.on('scope.conversation.updated', (data: unknown) => {
    if (!data || typeof data !== 'object') return;
    const event = data as ScopeConversationUpdate;
    if (
      typeof event.project_id !== 'string' ||
      !Number.isInteger(event.revision) ||
      typeof event.event_id !== 'string'
    )
      return;
    inbox.scheduleRefresh();
    if (event.kind === 'read') return;
    emit(event);
    if (
      client &&
      event.kind === 'comment' &&
      event.actor_kind === 'professional' &&
      !isScopeConversationFocused(
        event.project_id,
        event.revision,
        event.change_request_id ?? undefined,
      )
    ) {
      $q.notify({
        message: t('notifications.scope.message.title', { project: event.project_name }),
        ...(event.actor_name ? { caption: event.actor_name } : {}),
        type: 'info',
        position: 'top-right',
        actions: [
          {
            label: t('scope.openConversation'),
            style: { color: 'inherit' },
            handler: () => {
              void router.push({
                path: event.change_request_id
                  ? `/client/change-requests/${event.change_request_id}`
                  : `/client/projects/${event.project_id}`,
                query: {
                  conversation: '1',
                  revision: String(event.revision),
                  message: event.event_id,
                  focus: String(Date.now()),
                },
              });
            },
          },
        ],
      });
    }
  });
  websocket.on('scope.conversation.resync', () => {
    inbox.scheduleRefresh();
    emit(null);
  });
  watch(websocket.isConnected, (connected) => {
    if (connected) {
      inbox.scheduleRefresh();
      emit(null);
    }
  });
  const refocus = () => {
    if (document.visibilityState === 'visible' && document.hasFocus()) {
      inbox.scheduleRefresh();
      emit(null);
    }
  };
  window.addEventListener('focus', refocus);
  document.addEventListener('visibilitychange', refocus);
  onUnmounted(() => {
    window.removeEventListener('focus', refocus);
    document.removeEventListener('visibilitychange', refocus);
  });
  return websocket;
}
