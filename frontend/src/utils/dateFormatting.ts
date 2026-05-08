import type { UserDateFormatPreference } from 'src/types/user';

type DateInput = string | number | Date;

interface DateParts {
  day: string;
  month: string;
  year: string;
  hour: string;
  minute: string;
}

export interface DateFormattingOptions {
  timezone?: string | null;
  dateFormat?: UserDateFormatPreference | null;
  includeTime?: boolean;
  fallback?: string;
}

export const DEFAULT_TIMEZONE = 'UTC';
export const DEFAULT_DATE_FORMAT: UserDateFormatPreference = 'DD/MM/YYYY';
const ISO_DATE_ONLY_PATTERN = /^(\d{4})-(\d{2})-(\d{2})$/;

const SUPPORTED_DATE_FORMATS = new Set<UserDateFormatPreference>([
  'DD/MM/YYYY',
  'MM/DD/YYYY',
  'YYYY/MM/DD',
]);

export function normalizeTimezone(timezone: string | null | undefined): string {
  const normalized = timezone?.trim();
  return normalized && normalized.length > 0 ? normalized : DEFAULT_TIMEZONE;
}

export function normalizeDateFormat(
  dateFormat: UserDateFormatPreference | null | undefined,
): UserDateFormatPreference {
  if (typeof dateFormat === 'string' && SUPPORTED_DATE_FORMATS.has(dateFormat)) {
    return dateFormat;
  }

  return DEFAULT_DATE_FORMAT;
}

function toDate(value: DateInput): Date | null {
  if (value instanceof Date) {
    return Number.isNaN(value.getTime()) ? null : value;
  }

  const parsed = new Date(value);
  return Number.isNaN(parsed.getTime()) ? null : parsed;
}

function getDateOnlyParts(value: DateInput): DateParts | null {
  if (typeof value !== 'string') {
    return null;
  }

  const match = ISO_DATE_ONLY_PATTERN.exec(value.trim());
  if (!match) {
    return null;
  }

  const [, year, month, day] = match;
  if (!year || !month || !day) {
    return null;
  }

  return {
    day,
    month,
    year,
    hour: '00',
    minute: '00',
  };
}

function getDateParts(date: Date, timezone: string, includeTime: boolean): DateParts | null {
  const options: Intl.DateTimeFormatOptions = {
    timeZone: timezone,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
  };

  if (includeTime) {
    options.hour = '2-digit';
    options.minute = '2-digit';
    options.hour12 = false;
    options.hourCycle = 'h23';
  }

  try {
    const formatter = new Intl.DateTimeFormat('en-US', options);
    const valuesByType = new Map<string, string>();

    for (const part of formatter.formatToParts(date)) {
      if (part.type !== 'literal') {
        valuesByType.set(part.type, part.value);
      }
    }

    const day = valuesByType.get('day');
    const month = valuesByType.get('month');
    const year = valuesByType.get('year');
    const hour = valuesByType.get('hour') ?? '00';
    const minute = valuesByType.get('minute') ?? '00';

    if (!day || !month || !year) {
      return null;
    }

    return { day, month, year, hour, minute };
  } catch {
    return null;
  }
}

function buildDatePart(parts: DateParts, dateFormat: UserDateFormatPreference): string {
  switch (dateFormat) {
    case 'MM/DD/YYYY':
      return `${parts.month}/${parts.day}/${parts.year}`;
    case 'YYYY/MM/DD':
      return `${parts.year}/${parts.month}/${parts.day}`;
    case 'DD/MM/YYYY':
    default:
      return `${parts.day}/${parts.month}/${parts.year}`;
  }
}

export function formatDateByPreferences(
  value: DateInput,
  options: DateFormattingOptions = {},
): string {
  const fallback = options.fallback ?? '';
  const dateFormat = normalizeDateFormat(options.dateFormat);
  const timezone = normalizeTimezone(options.timezone);
  const includeTime = options.includeTime === true;
  const dateOnlyParts = getDateOnlyParts(value);

  if (dateOnlyParts) {
    const datePart = buildDatePart(dateOnlyParts, dateFormat);
    return includeTime ? `${datePart} ${dateOnlyParts.hour}:${dateOnlyParts.minute}` : datePart;
  }

  const parsedDate = toDate(value);

  if (!parsedDate) {
    return fallback;
  }

  const parts =
    getDateParts(parsedDate, timezone, includeTime) ||
    getDateParts(parsedDate, DEFAULT_TIMEZONE, includeTime);

  if (!parts) {
    return fallback;
  }

  const datePart = buildDatePart(parts, dateFormat);
  return includeTime ? `${datePart} ${parts.hour}:${parts.minute}` : datePart;
}

export function getDateInputMask(dateFormat: UserDateFormatPreference): string {
  const normalized = normalizeDateFormat(dateFormat);
  return normalized === 'YYYY/MM/DD' ? '####/##/##' : '##/##/####';
}

export function getQDateMask(dateFormat: UserDateFormatPreference): UserDateFormatPreference {
  return normalizeDateFormat(dateFormat);
}

function normalizeDateTokens(value: string, dateFormat: UserDateFormatPreference) {
  const rawTokens = value
    .trim()
    .split('/')
    .map((token) => token.trim());

  if (rawTokens.length !== 3) {
    return null;
  }

  let day = '';
  let month = '';
  let year = '';

  if (dateFormat === 'DD/MM/YYYY') {
    // [day, month, year] = rawTokens;
    day = rawTokens[0]!;
    month = rawTokens[1]!;
    year = rawTokens[2]!;
  } else if (dateFormat === 'MM/DD/YYYY') {
    month = rawTokens[0]!;
    day = rawTokens[1]!;
    year = rawTokens[2]!;
  } else {
    year = rawTokens[0]!;
    month = rawTokens[1]!;
    day = rawTokens[2]!;
  }

  if (!day || !month || !year) {
    return null;
  }

  if (!/^\d+$/.test(day) || !/^\d+$/.test(month) || !/^\d+$/.test(year)) {
    return null;
  }

  const paddedDay = day.padStart(2, '0');
  const paddedMonth = month.padStart(2, '0');

  if (year.length !== 4 || paddedDay.length !== 2 || paddedMonth.length !== 2) {
    return null;
  }

  const dayNumber = Number(paddedDay);
  const monthNumber = Number(paddedMonth);
  const yearNumber = Number(year);

  if (
    !Number.isInteger(dayNumber) ||
    !Number.isInteger(monthNumber) ||
    !Number.isInteger(yearNumber) ||
    monthNumber < 1 ||
    monthNumber > 12 ||
    dayNumber < 1 ||
    dayNumber > 31
  ) {
    return null;
  }

  const candidate = new Date(Date.UTC(yearNumber, monthNumber - 1, dayNumber, 12, 0, 0));
  if (
    candidate.getUTCFullYear() !== yearNumber ||
    candidate.getUTCMonth() + 1 !== monthNumber ||
    candidate.getUTCDate() !== dayNumber
  ) {
    return null;
  }

  return {
    day: paddedDay,
    month: paddedMonth,
    year,
  };
}

export function parseDisplayDateToIso(
  value: string,
  dateFormat: UserDateFormatPreference,
): string | null {
  const dateOnly = parseDisplayDateToDateOnly(value, dateFormat);

  return dateOnly ? `${dateOnly}T12:00:00.000Z` : null;
}

export function parseDisplayDateToDateOnly(
  value: string,
  dateFormat: UserDateFormatPreference,
): string | null {
  const isoDateOnly = getDateOnlyParts(value);

  if (isoDateOnly) {
    return `${isoDateOnly.year}-${isoDateOnly.month}-${isoDateOnly.day}`;
  }

  const normalized = normalizeDateFormat(dateFormat);
  const tokens = normalizeDateTokens(value, normalized);

  if (!tokens) {
    return null;
  }

  return `${tokens.year}-${tokens.month}-${tokens.day}`;
}

export function formatIsoDateForInput(
  value: string | null,
  options: Pick<DateFormattingOptions, 'timezone' | 'dateFormat'> = {},
): string {
  if (!value) {
    return '';
  }

  return formatDateByPreferences(value, {
    timezone: options.timezone!,
    dateFormat: options.dateFormat!,
    includeTime: false,
    fallback: '',
  });
}
