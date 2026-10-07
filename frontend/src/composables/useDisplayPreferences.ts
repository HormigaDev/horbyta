import { computed, reactive, getCurrentInstance } from 'vue';
import { useRoute } from 'vue-router';
import { useUserPreferencesStore } from 'src/stores/userPreferences';
import { DEFAULT_DATE_FORMAT, DEFAULT_TIMEZONE } from 'src/utils/dateFormatting';
import {
  DEFAULT_DECIMAL_QUANTITY,
  DEFAULT_DECIMAL_SEPARATOR,
  DEFAULT_THOUSAND_SEPARATOR,
} from 'src/utils/numberFormatting';

/** Portal presentation never consumes the professional account's regional settings. */
export function useDisplayPreferences() {
  const route = getCurrentInstance() ? useRoute() : null;
  const professional = useUserPreferencesStore();
  const portal = computed(() => {
    const path = route?.path ?? window.location.pathname;
    return path === '/client' || path.startsWith('/client/');
  });
  return reactive({
    timezone: computed(() => (portal.value ? DEFAULT_TIMEZONE : professional.timezone)),
    dateFormat: computed(() => (portal.value ? DEFAULT_DATE_FORMAT : professional.dateFormat)),
    thousandSeparator: computed(() =>
      portal.value ? DEFAULT_THOUSAND_SEPARATOR : professional.thousandSeparator,
    ),
    decimalSeparator: computed(() =>
      portal.value ? DEFAULT_DECIMAL_SEPARATOR : professional.decimalSeparator,
    ),
    decimalQuantity: computed(() =>
      portal.value ? DEFAULT_DECIMAL_QUANTITY : professional.decimalQuantity,
    ),
  });
}
