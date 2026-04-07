import { computed, ref } from 'vue';

type ToolStatus = 'running' | 'completed' | 'error';

export interface ToolBadge {
  kind: string;
  label: string;
  route?: string;
  id?: string;
  count?: number;
}

export interface ToolEvent {
  key: string;
  status: ToolStatus;
  toolName: string;
  message: string;
  badge?: ToolBadge;
  createdAt: string;
}

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null;

interface UseChatbotToolActivityOptions {
  t: (key: string) => string;
}

export function useChatbotToolActivity(options: UseChatbotToolActivityOptions) {
  const toolEvents = ref<ToolEvent[]>([]);
  const aiRuntimeStatus = ref<string>('idle');
  const aiRuntimeMessage = ref('');

  const workingStatusMessage = computed(() => {
    if (aiRuntimeStatus.value === 'processing' && aiRuntimeMessage.value) {
      return aiRuntimeMessage.value;
    }

    const running = toolEvents.value.find((event) => event.status === 'running');
    return running?.message ?? '';
  });

  const visibleToolEvents = computed(() => toolEvents.value.slice(-8));

  const startRun = () => {
    toolEvents.value = [];
    aiRuntimeStatus.value = 'processing';
    aiRuntimeMessage.value = options.t('chatbot.panel.thinking');
  };

  const markCompleted = () => {
    aiRuntimeStatus.value = 'idle';
    aiRuntimeMessage.value = '';
  };

  const markError = () => {
    aiRuntimeStatus.value = 'error';
  };

  const applyAiStatusPayload = (payload: unknown) => {
    if (!isRecord(payload)) {
      return;
    }

    if (typeof payload.status === 'string') {
      aiRuntimeStatus.value = payload.status;
    }

    if (typeof payload.message === 'string') {
      aiRuntimeMessage.value = payload.message;
    }
  };

  const applyToolStatusPayload = (payload: unknown) => {
    if (!isRecord(payload)) {
      return;
    }

    const toolCallId =
      typeof payload.tool_call_id === 'string' ? payload.tool_call_id : crypto.randomUUID();
    const toolName = typeof payload.tool_name === 'string' ? payload.tool_name : 'tool';
    const rawStatus = typeof payload.status === 'string' ? payload.status : 'running';
    const status: ToolStatus =
      rawStatus === 'completed' || rawStatus === 'error' ? rawStatus : 'running';

    let message = `${options.t('chatbot.panel.toolWorking')} ${toolName}`;
    let badge: ToolBadge | null = null;

    const details = isRecord(payload.details) ? payload.details : null;
    if (details && typeof details.message === 'string') {
      message = details.message;
    }

    if (details && isRecord(details.badge)) {
      const badgeObj = details.badge;
      if (typeof badgeObj.label === 'string' && typeof badgeObj.kind === 'string') {
        const nextBadge: ToolBadge = {
          label: badgeObj.label,
          kind: badgeObj.kind,
        };

        if (typeof badgeObj.route === 'string') {
          nextBadge.route = badgeObj.route;
        }

        if (typeof badgeObj.id === 'string') {
          nextBadge.id = badgeObj.id;
        }

        if (typeof badgeObj.count === 'number') {
          nextBadge.count = badgeObj.count;
        }

        badge = nextBadge;
      }
    }

    const nextEvent: ToolEvent = {
      key: toolCallId,
      status,
      toolName,
      message,
      createdAt: new Date().toISOString(),
    };

    if (badge) {
      nextEvent.badge = badge;
    }

    const existingIndex = toolEvents.value.findIndex((event) => event.key === toolCallId);
    if (existingIndex >= 0) {
      toolEvents.value[existingIndex] = nextEvent;
      return;
    }

    toolEvents.value.push(nextEvent);
  };

  return {
    toolEvents,
    aiRuntimeStatus,
    aiRuntimeMessage,
    workingStatusMessage,
    visibleToolEvents,
    startRun,
    markCompleted,
    markError,
    applyAiStatusPayload,
    applyToolStatusPayload,
  };
}
