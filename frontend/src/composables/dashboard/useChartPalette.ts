import { computed, onMounted, onUnmounted, ref, watch, type ComputedRef } from 'vue';
import { storeToRefs } from 'pinia';

import { useThemeStore } from 'src/stores/theme';

export interface ChartPalette {
  palette: string[];
  axis: string;
  grid: string;
  text: string;
  tooltipBg: string;
  primary: string;
}

const readCssVar = (name: string, fallback: string): string => {
  if (typeof window === 'undefined') {
    return fallback;
  }
  const value = getComputedStyle(document.body).getPropertyValue(name).trim();
  return value.length > 0 ? value : fallback;
};

const buildPalette = (): ChartPalette => ({
  palette: [
    readCssVar('--chart-color-1', '#5865f2'),
    readCssVar('--chart-color-2', '#9b7cf2'),
    readCssVar('--chart-color-3', '#22c7a9'),
    readCssVar('--chart-color-4', '#f2b15c'),
    readCssVar('--chart-color-5', '#f26a6a'),
    readCssVar('--chart-color-6', '#4ec4e6'),
  ],
  axis: readCssVar('--chart-axis', '#5a5a5a'),
  grid: readCssVar('--chart-grid', '#3a3a3a'),
  text: readCssVar('--chart-text', '#c8ccd1'),
  tooltipBg: readCssVar('--chart-tooltip-bg', 'rgba(30,30,30,0.95)'),
  primary: readCssVar('--q-primary', '#5865f2'),
});

export function useChartPalette(): ComputedRef<ChartPalette> {
  const themeStore = useThemeStore();
  const { isDark } = storeToRefs(themeStore);
  const paletteKey = ref(0);
  const observer = ref<MutationObserver | null>(null);

  onMounted(() => {
    if (typeof window === 'undefined') {
      return;
    }
    const mutationObserver = new MutationObserver(() => {
      paletteKey.value += 1;
    });
    mutationObserver.observe(document.body, {
      attributes: true,
      attributeFilter: ['data-theme'],
    });
    observer.value = mutationObserver;
  });

  onUnmounted(() => {
    observer.value?.disconnect();
    observer.value = null;
  });

  watch(isDark, () => {
    paletteKey.value += 1;
  });

  return computed<ChartPalette>(() => {
    void paletteKey.value;
    return buildPalette();
  });
}
