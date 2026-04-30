import { defineStore } from 'pinia';
import { ref } from 'vue';
import { Dark } from 'quasar';

export const useThemeStore = defineStore('theme', () => {
  const defaultIsDark = true;
  const isDark = ref(defaultIsDark);

  const setInMemoryThemePreference = (value: boolean) => {
    isDark.value = value;
  };

  const toggleTheme = (val?: boolean) => {
    setInMemoryThemePreference(val !== undefined ? val : !isDark.value);
    applyTheme();
  };

  const setThemePreference = (value: boolean) => {
    setInMemoryThemePreference(value);
    applyTheme(value);
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
    setThemePreference,
    applyTheme,
  };
});
