import type { UserNumberFormatPreference } from 'src/types/user';

export type NumberInput = string | number | null | undefined;

export const DEFAULT_THOUSAND_SEPARATOR = ',';
export const DEFAULT_DECIMAL_SEPARATOR = '.';
export const DEFAULT_DECIMAL_QUANTITY = 2;

const SUPPORTED_SEPARATORS = new Set([',', '.']);

const toFiniteNumber = (value: NumberInput): number | null => {
  if (value === null || value === undefined || value === '') {
    return null;
  }

  const parsed = typeof value === 'number' ? value : Number(value);
  return Number.isFinite(parsed) ? parsed : null;
};

export function normalizeDecimalQuantity(value: number | null | undefined): number {
  if (!Number.isFinite(value)) {
    return DEFAULT_DECIMAL_QUANTITY;
  }

  const normalized = Math.trunc(Number(value));
  if (normalized < 1) return 1;
  if (normalized > 4) return 4;
  return normalized;
}

export function normalizeNumberFormat(
  preferences: Partial<UserNumberFormatPreference> | null | undefined,
): UserNumberFormatPreference {
  const thousandSeparator = preferences?.thousand_separator;
  const decimalSeparator = preferences?.decimal_separator;

  if (
    typeof thousandSeparator === 'string' &&
    typeof decimalSeparator === 'string' &&
    SUPPORTED_SEPARATORS.has(thousandSeparator) &&
    SUPPORTED_SEPARATORS.has(decimalSeparator) &&
    thousandSeparator !== decimalSeparator
  ) {
    return {
      thousand_separator: thousandSeparator,
      decimal_separator: decimalSeparator,
      decimal_quantity: normalizeDecimalQuantity(preferences?.decimal_quantity),
    };
  }

  return {
    thousand_separator: DEFAULT_THOUSAND_SEPARATOR,
    decimal_separator: DEFAULT_DECIMAL_SEPARATOR,
    decimal_quantity: normalizeDecimalQuantity(preferences?.decimal_quantity),
  };
}

export function formatNumberByPreferences(
  value: NumberInput,
  preferences: Partial<UserNumberFormatPreference> | null | undefined,
  fallback = '—',
): string {
  const parsed = toFiniteNumber(value);
  if (parsed === null) {
    return fallback;
  }

  const normalized = normalizeNumberFormat(preferences);
  const fixed = parsed.toFixed(normalized.decimal_quantity);
  const [integerPart = '0', decimalPart = ''] = fixed.split('.');
  const sign = integerPart.startsWith('-') ? '-' : '';
  const unsignedInteger = sign ? integerPart.slice(1) : integerPart;
  const groupedInteger = unsignedInteger.replace(
    /\B(?=(\d{3})+(?!\d))/g,
    normalized.thousand_separator,
  );

  return `${sign}${groupedInteger}${normalized.decimal_separator}${decimalPart}`;
}

export function formatCurrencyByPreferences(
  value: NumberInput,
  currency: string | null | undefined,
  preferences: Partial<UserNumberFormatPreference> | null | undefined,
  fallback = '—',
): string {
  const formatted = formatNumberByPreferences(value, preferences, fallback);
  if (formatted === fallback) {
    return fallback;
  }

  return `${currency || ''} ${formatted}`.trim();
}
