import { defineStore } from 'pinia';
import { ref } from 'vue';
import { Dark } from 'quasar';

export const useThemeStore = defineStore('theme', () => {
  const themeStorageKey = 'isDark';
  const defaultIsDark = true;
  const isDark = ref(
    localStorage.getItem(themeStorageKey) !== null
      ? localStorage.getItem(themeStorageKey) === 'true'
      : defaultIsDark,
  );

  const persistThemePreference = (value: boolean) => {
    isDark.value = value;
    localStorage.setItem(themeStorageKey, String(value));
  };

  const toggleTheme = (val?: boolean) => {
    persistThemePreference(val !== undefined ? val : !isDark.value);
    applyTheme();
  };

  const setThemePreference = (value: boolean) => {
    persistThemePreference(value);
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
