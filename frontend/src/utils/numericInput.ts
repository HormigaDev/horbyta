export interface NumericTextOptions {
  onlyIntegers?: boolean;
  decimalPlaces?: number;
  keepIncompleteDecimals?: boolean;
}

const clampDecimalPlaces = (value: number | undefined): number => {
  if (value === undefined) return 2;
  if (value < 0) return 0;
  return Math.floor(value);
};

const normalizeIntegerPart = (value: string): string => {
  const digitsOnly = value.replace(/\D/g, '');

  if (!digitsOnly) {
    return '0';
  }

  const withoutLeadingZeros = digitsOnly.replace(/^0+(?=\d)/, '');
  return withoutLeadingZeros || '0';
};

export const normalizeNumericText = (
  rawValue: string,
  options: NumericTextOptions = {},
): string => {
  const onlyIntegers = options.onlyIntegers ?? false;
  const decimalPlaces = clampDecimalPlaces(options.decimalPlaces);
  const keepIncompleteDecimals = options.keepIncompleteDecimals ?? false;

  if (!rawValue) return '';

  if (onlyIntegers) {
    const digitsOnly = rawValue.replace(/\D/g, '');
    if (!digitsOnly) return '';
    return normalizeIntegerPart(digitsOnly);
  }

  const withComma = rawValue.replace(/\./g, ',');
  const cleaned = withComma.replace(/[^0-9,]/g, '');
  const separatorIndex = cleaned.indexOf(',');

  if (separatorIndex === -1) {
    return normalizeIntegerPart(cleaned);
  }

  const integerPart = normalizeIntegerPart(cleaned.slice(0, separatorIndex));
  let decimalPart = cleaned.slice(separatorIndex + 1).replace(/,/g, '');
  const hadTrailingSeparator = cleaned.endsWith(',');

  if (decimalPlaces === 0) {
    return integerPart;
  }

  if (decimalPart.length > decimalPlaces) {
    decimalPart = decimalPart.slice(0, decimalPlaces);
  }

  if (!decimalPart.length) {
    return hadTrailingSeparator ? `${integerPart},` : integerPart;
  }

  const hasNonZeroDecimalDigit = /[1-9]/.test(decimalPart);

  if (hasNonZeroDecimalDigit) {
    decimalPart = decimalPart.replace(/0+$/, '');
  } else if (!keepIncompleteDecimals) {
    decimalPart = '';
  }

  if (!decimalPart.length) {
    return integerPart;
  }

  return `${integerPart},${decimalPart}`;
};

export const parseNumericText = (value: string, options: NumericTextOptions = {}): number => {
  const normalized = normalizeNumericText(value, {
    ...options,
    keepIncompleteDecimals: false,
  });

  if (!normalized) {
    return Number.NaN;
  }

  if (options.onlyIntegers) {
    if (!/^\d+$/.test(normalized)) {
      return Number.NaN;
    }

    return Number.parseInt(normalized, 10);
  }

  const numericValue = Number(normalized.replace(',', '.'));
  return Number.isFinite(numericValue) ? numericValue : Number.NaN;
};
