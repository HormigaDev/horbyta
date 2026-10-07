import type { AppearancePreference, UserThemePreference } from 'src/types/user';

export interface AppearanceState extends AppearancePreference {
  mode: UserThemePreference;
}
export const defaultAppearance = (): AppearanceState => ({
  mode: 'system',
  theme_id: 'orbit',
  density: 'comfortable',
  reduced_motion: false,
});
export function normalizeAppearance(value: unknown): AppearanceState {
  const candidate = value && typeof value === 'object' ? (value as Partial<AppearanceState>) : {};
  return {
    mode: ['light', 'dark', 'system'].includes(candidate.mode ?? '') ? candidate.mode! : 'system',
    theme_id: ['orbit', 'linen', 'forest'].includes(candidate.theme_id ?? '')
      ? candidate.theme_id!
      : 'orbit',
    density: candidate.density === 'compact' ? 'compact' : 'comfortable',
    reduced_motion: candidate.reduced_motion === true,
  };
}
export function readAppearance(key: string): AppearanceState {
  try {
    const data: unknown = JSON.parse(localStorage.getItem(key) ?? 'null');
    if (
      data &&
      typeof data === 'object' &&
      'version' in data &&
      data.version === 1 &&
      'value' in data
    ) {
      return normalizeAppearance(data.value);
    }
  } catch {
    /* Missing, corrupt or unavailable storage uses system defaults. */
  }
  return defaultAppearance();
}
export function writeAppearance(key: string, value: AppearanceState) {
  try {
    localStorage.setItem(key, JSON.stringify({ version: 1, value }));
  } catch {
    /* Optional cache. */
  }
}
export function appearanceKey(scope: string) {
  return `horbyta:appearance:v1:${scope}`;
}
