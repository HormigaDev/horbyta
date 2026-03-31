import { defineStore } from 'pinia';
import { ref } from 'vue';
import { Dark } from 'quasar';

export const useThemeStore = defineStore('theme', () => {
  const defaultIsDark = true;
  const isDark = ref(
    localStorage.getItem('isDark') !== null
      ? localStorage.getItem('isDark') === 'true'
      : defaultIsDark,
  );

  const toggleTheme = (val?: boolean) => {
    isDark.value = val !== undefined ? val : !isDark.value;
    localStorage.setItem('isDark', String(isDark.value));
    applyTheme();
  };

  const applyTheme = (forceDark?: boolean) => {
    const themeToApply = forceDark !== undefined ? forceDark : isDark.value;

    // Sincronizar con el sistema interno de Quasar
    Dark.set(themeToApply);

    if (themeToApply) {
      document.body.setAttribute('data-theme', 'dark');
    } else {
      document.body.setAttribute('data-theme', 'light');
    }
  };

  return {
    isDark,
    toggleTheme,
    applyTheme,
  };
});
