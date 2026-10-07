import { computed } from 'vue';
import { CURRENCY_SYMBOLS } from 'src/data/constants';
import { useDisplayPreferences } from 'src/composables/useDisplayPreferences';
import {
  formatCurrencyByPreferences,
  formatNumberByPreferences,
  normalizeNumberFormat,
  type NumberInput,
} from 'src/utils/numberFormatting';

export function useNumberFormatter() {
  const preferencesStore = useDisplayPreferences();

  const numberFormat = computed(() =>
    normalizeNumberFormat({
      thousand_separator: preferencesStore.thousandSeparator,
      decimal_separator: preferencesStore.decimalSeparator,
      decimal_quantity: preferencesStore.decimalQuantity,
    }),
  );

  const formatNumber = (value: NumberInput, fallback = '—') =>
    formatNumberByPreferences(value, numberFormat.value, fallback);

  const formatCurrency = (
    value: NumberInput,
    currencyType: string | null | undefined,
    fallback = '—',
    useSymbol = true,
  ) => {
    const currency = useSymbol
      ? (CURRENCY_SYMBOLS[currencyType || ''] ?? currencyType)
      : currencyType;

    return formatCurrencyByPreferences(value, currency, numberFormat.value, fallback);
  };

  return {
    numberFormat,
    formatNumber,
    formatCurrency,
  };
}
