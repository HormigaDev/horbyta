import { ref } from 'vue';

const SCROLL_RETRY_DELAYS: readonly number[] = [0, 40, 120, 260];

export function useChatbotAutoScroll() {
  const messagesContainer = ref<HTMLElement | null>(null);
  const messagesBottomAnchor = ref<HTMLElement | null>(null);
  const pendingScrollTimeoutIds = ref<number[]>([]);
  const pendingScrollAnimationFrameId = ref<number | null>(null);

  const isScrollableElement = (element: HTMLElement): boolean => {
    const style = window.getComputedStyle(element);
    const overflowY = style.overflowY;
    return (
      (overflowY === 'auto' || overflowY === 'scroll' || overflowY === 'overlay') &&
      element.scrollHeight > element.clientHeight
    );
  };

  const collectScrollableAncestors = (element: HTMLElement | null): HTMLElement[] => {
    const scrollables: HTMLElement[] = [];
    let current = element?.parentElement ?? null;

    while (current) {
      if (isScrollableElement(current)) {
        scrollables.push(current);
      }

      if (current.classList.contains('chatbot-shell')) {
        break;
      }

      current = current.parentElement;
    }

    return scrollables;
  };

  const scrollElementToBottom = (element: HTMLElement) => {
    element.scrollTop = element.scrollHeight;
    element.scrollTo({ top: element.scrollHeight, behavior: 'auto' });
  };

  const clearScheduledScrollWork = () => {
    if (pendingScrollAnimationFrameId.value !== null) {
      window.cancelAnimationFrame(pendingScrollAnimationFrameId.value);
      pendingScrollAnimationFrameId.value = null;
    }

    pendingScrollTimeoutIds.value.forEach((timeoutId) => {
      window.clearTimeout(timeoutId);
    });

    pendingScrollTimeoutIds.value = [];
  };

  const scrollToBottom = () => {
    const anchor = messagesBottomAnchor.value;
    if (anchor) {
      anchor.scrollIntoView({ block: 'end', inline: 'nearest', behavior: 'auto' });
    }

    const targets = new Set<HTMLElement>();

    if (messagesContainer.value) {
      targets.add(messagesContainer.value);
    }

    const ancestorSource = anchor ?? messagesContainer.value;
    collectScrollableAncestors(ancestorSource).forEach((element) => {
      targets.add(element);
    });

    targets.forEach((element) => {
      scrollElementToBottom(element);
    });
  };

  const scheduleScrollToBottom = () => {
    clearScheduledScrollWork();

    SCROLL_RETRY_DELAYS.forEach((delay) => {
      const timeoutId = window.setTimeout(() => {
        pendingScrollAnimationFrameId.value = window.requestAnimationFrame(() => {
          scrollToBottom();
          pendingScrollAnimationFrameId.value = null;
        });
      }, delay);

      pendingScrollTimeoutIds.value.push(timeoutId);
    });
  };

  return {
    messagesContainer,
    messagesBottomAnchor,
    clearScheduledScrollWork,
    scrollToBottom,
    scheduleScrollToBottom,
  };
}
