export interface NumericTextOptions {
  onlyIntegers?: boolean;
  decimalPlaces?: number;
  keepIncompleteDecimals?: boolean;
  decimalSeparator?: ',' | '.';
  thousandSeparator?: ',' | '.';
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

const stripGroupedThousands = (value: string, thousandSeparator: ',' | '.'): string => {
  let output = '';

  for (let index = 0; index < value.length; index += 1) {
    const char = value[index];

    if (char !== thousandSeparator) {
      output += char;
      continue;
    }

    const previous = value[index - 1] ?? '';
    const nextGroup = value.slice(index + 1, index + 4);
    const afterGroup = value[index + 4] ?? '';
    const isGroupedThousands =
      /\d/.test(previous) &&
      /^\d{3}$/.test(nextGroup) &&
      (afterGroup === '' || afterGroup === thousandSeparator || !/\d/.test(afterGroup));

    if (!isGroupedThousands) {
      output += char;
    }
  }

  return output;
};

export const normalizeNumericText = (
  rawValue: string,
  options: NumericTextOptions = {},
): string => {
  const onlyIntegers = options.onlyIntegers ?? false;
  const decimalPlaces = clampDecimalPlaces(options.decimalPlaces);
  const keepIncompleteDecimals = options.keepIncompleteDecimals ?? false;
  const decimalSeparator = options.decimalSeparator === '.' ? '.' : ',';
  const thousandSeparator =
    options.thousandSeparator === ',' || options.thousandSeparator === '.'
      ? options.thousandSeparator
      : null;

  if (!rawValue) return '';

  if (onlyIntegers) {
    const digitsOnly = rawValue.replace(/\D/g, '');
    if (!digitsOnly) return '';
    return normalizeIntegerPart(digitsOnly);
  }

  const withoutThousands = thousandSeparator
    ? stripGroupedThousands(rawValue, thousandSeparator)
    : rawValue;
  const normalizedSeparator = withoutThousands.replace(/[.,]/g, decimalSeparator);
  const separatorPattern = decimalSeparator === '.' ? /[^0-9.]/g : /[^0-9,]/g;
  const cleaned = normalizedSeparator.replace(separatorPattern, '');
  const separatorIndex = cleaned.indexOf(decimalSeparator);

  if (separatorIndex === -1) {
    return normalizeIntegerPart(cleaned);
  }

  const integerPart = normalizeIntegerPart(cleaned.slice(0, separatorIndex));
  const repeatedSeparatorPattern = decimalSeparator === '.' ? /\./g : /,/g;
  let decimalPart = cleaned.slice(separatorIndex + 1).replace(repeatedSeparatorPattern, '');
  const hadTrailingSeparator = cleaned.endsWith(decimalSeparator);

  if (decimalPlaces === 0) {
    return integerPart;
  }

  if (decimalPart.length > decimalPlaces) {
    decimalPart = decimalPart.slice(0, decimalPlaces);
  }

  if (!decimalPart.length) {
    return hadTrailingSeparator ? `${integerPart}${decimalSeparator}` : integerPart;
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

  return `${integerPart}${decimalSeparator}${decimalPart}`;
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
