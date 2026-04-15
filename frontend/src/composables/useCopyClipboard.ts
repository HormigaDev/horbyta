import { ref } from 'vue';

export function useCopyClipboard(timeout = 1500) {
  const isCopied = ref(false);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text).catch(() => null);
    isCopied.value = true;

    setTimeout(() => (isCopied.value = false), timeout);
  };

  return {
    isCopied,
    copyToClipboard,
  };
}
