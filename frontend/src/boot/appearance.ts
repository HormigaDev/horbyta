import { defineBoot } from '#q-app/wrappers';
import { useThemeStore } from 'src/stores/theme';
import { cycleDialogFocus } from 'src/utils/dialogFocus';

export default defineBoot(({ app }) => {
  useThemeStore().applyTheme();
  document.addEventListener('keydown', cycleDialogFocus);
  const cleanup = () => document.removeEventListener('keydown', cycleDialogFocus);
  app.onUnmount(cleanup);
  import.meta.hot?.dispose(cleanup);
});
