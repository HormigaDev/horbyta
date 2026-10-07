import { defineStore } from 'pinia';
import { computed, onScopeDispose, ref } from 'vue';
import { Dark } from 'quasar';
import type { UserPreferences } from 'src/types/user';
import {
  appearanceKey,
  defaultAppearance,
  normalizeAppearance,
  readAppearance,
  writeAppearance,
  type AppearanceState,
} from 'src/utils/appearance';

export const useThemeStore = defineStore('theme', () => {
  const scope = ref('anonymous');
  const accountId = ref<string | null>(null);
  const appearance = ref(readAppearance(appearanceKey(scope.value)));
  const confirmed = ref<AppearanceState>({ ...appearance.value });
  const system = window.matchMedia('(prefers-color-scheme: dark)');
  const systemDark = ref(system.matches);
  const isDark = computed(() =>
    appearance.value.mode === 'system' ? systemDark.value : appearance.value.mode === 'dark',
  );
  const themeMode = computed(() => appearance.value.mode);
  const resolvedMode = computed(() => (isDark.value ? 'dark' : 'light'));

  function applyTheme() {
    const root = document.documentElement;
    root.dataset.theme = resolvedMode.value;
    root.dataset.themeId = appearance.value.theme_id;
    root.dataset.density = appearance.value.density;
    root.dataset.motion = appearance.value.reduced_motion ? 'reduced' : 'system';
    root.style.colorScheme = resolvedMode.value;
    const chromeColor = getComputedStyle(root).getPropertyValue('--surface-nav').trim();
    document.querySelector('meta[name="theme-color"]')?.setAttribute('content', chromeColor);
    Dark.set(isDark.value);
  }
  function selectScope(next: string) {
    if (scope.value === next) return;
    scope.value = next;
    appearance.value = readAppearance(appearanceKey(next));
    confirmed.value = { ...appearance.value };
    applyTheme();
  }
  function bindAccount(id: string | null) {
    accountId.value = id;
    selectScope(id ? `user:${id}` : 'anonymous');
  }
  function setPortalRoute(portal: boolean) {
    selectScope(portal ? 'portal' : accountId.value ? `user:${accountId.value}` : 'anonymous');
  }
  function preview(value: AppearanceState) {
    appearance.value = normalizeAppearance(value);
    applyTheme();
  }
  function cancelPreview() {
    preview(confirmed.value);
  }
  function commit(value: AppearanceState) {
    preview(value);
    confirmed.value = { ...appearance.value };
    writeAppearance(appearanceKey(scope.value), confirmed.value);
  }
  function applyPreferences(preferences: UserPreferences | null | undefined) {
    commit(normalizeAppearance({ ...preferences?.appearance, mode: preferences?.theme }));
  }
  const onSystemChange = (event: MediaQueryListEvent) => {
    systemDark.value = event.matches;
    applyTheme();
  };
  system.addEventListener('change', onSystemChange);
  onScopeDispose(() => system.removeEventListener('change', onSystemChange));
  applyTheme();
  return {
    appearance,
    confirmed,
    isDark,
    themeMode,
    resolvedMode,
    applyTheme,
    bindAccount,
    setPortalRoute,
    preview,
    cancelPreview,
    commit,
    applyPreferences,
    defaultAppearance,
  };
});
