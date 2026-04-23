import type { ComposerTranslation } from 'vue-i18n';

export const APP_VERSION = 'v1.0.0';

export const COMMON_TIMEZONES = [
  'UTC',
  'America/Mexico_City',
  'America/Bogota',
  'America/Lima',
  'America/Santiago',
  'America/Argentina/Buenos_Aires',
  'America/Sao_Paulo',
  'America/New_York',
  'America/Los_Angeles',
  'Europe/Madrid',
  'Europe/London',
  'Europe/Lisbon',
  'Europe/Paris',
  'Europe/Berlin',
  'Asia/Tokyo',
  'Asia/Dubai',
  'Asia/Shanghai',
  'Australia/Sydney',
  'Africa/Johannesburg',
] as const;

export const EMAIL_REGEX = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;

export const CLIENT_PASSWORD_REGEX = /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[^A-Za-z\d]).{8,}$/;

export const CURRENCY_TYPE_OPTIONS = [
  { label: '🇺🇸 USD', value: 'USD' },
  { label: '🇪🇺 EUR', value: 'EUR' },
  { label: '🇯🇵 JPY', value: 'JPY' },
  { label: '🇬🇧 GBP', value: 'GBP' },
  { label: '🇨🇭 CHF', value: 'CHF' },
  { label: '🇨🇦 CAD', value: 'CAD' },
  { label: '🇦🇺 AUD', value: 'AUD' },
  { label: '🇳🇿 NZD', value: 'NZD' },
  { label: '🇲🇽 MXN', value: 'MXN' },
  { label: '🇧🇷 BRL', value: 'BRL' },
  { label: '🇦🇷 ARS', value: 'ARS' },
  { label: '🇨🇱 CLP', value: 'CLP' },
  { label: '🇨🇴 COP', value: 'COP' },
  { label: '🇵🇪 PEN', value: 'PEN' },
  { label: '🇺🇾 UYU', value: 'UYU' },
  { label: '🇨🇳 CNY', value: 'CNY' },
  { label: '🇭🇰 HKD', value: 'HKD' },
  { label: '🇸🇬 SGD', value: 'SGD' },
  { label: '🇰🇷 KRW', value: 'KRW' },
  { label: '🇮🇳 INR', value: 'INR' },
  { label: '🇹🇭 THB', value: 'THB' },
  { label: '🇸🇪 SEK', value: 'SEK' },
  { label: '🇳🇴 NOK', value: 'NOK' },
  { label: '🇩🇰 DKK', value: 'DKK' },
  { label: '🇵🇱 PLN', value: 'PLN' },
  { label: '🇨🇿 CZK', value: 'CZK' },
  { label: '🇭🇺 HUF', value: 'HUF' },
  { label: '🇦🇪 AED', value: 'AED' },
  { label: '🇸🇦 SAR', value: 'SAR' },
  { label: '🇿🇦 ZAR', value: 'ZAR' },
  { label: '🇪🇬 EGP', value: 'EGP' },
  { label: '🇳🇬 NGN', value: 'NGN' },
];

export const CURRENCY_SYMBOLS: Record<string, string> = {
  USD: '$',
  EUR: '€',
  JPY: '¥',
  GBP: '£',
  CHF: 'CHF',
  CAD: 'C$',
  AUD: 'A$',
  NZD: 'NZ$',
  MXN: '$',
  BRL: 'R$',
  ARS: '$',
  CLP: '$',
  COP: '$',
  PEN: 'S/',
  UYU: '$U',
  CNY: '¥',
  HKD: 'HK$',
  SGD: 'S$',
  KRW: '₩',
  INR: '₹',
  THB: '฿',
  SEK: 'kr',
  NOK: 'kr',
  DKK: 'kr',
  PLN: 'zł',
  CZK: 'Kč',
  HUF: 'Ft',
  AED: 'د.إ',
  SAR: '﷼',
  ZAR: 'R',
  EGP: '£',
  NGN: '₦',
};

export const getChangeRequestsDeclineReasons = (t: ComposerTranslation) => {
  return [
    { label: t('changeRequests.decline_reason.budget_exceeded'), value: 'budget_exceeded' },
    { label: t('changeRequests.decline_reason.roi_not_justified'), value: 'roi_not_justified' },
    { label: t('changeRequests.decline_reason.schedule_too_long'), value: 'schedule_too_long' },
    {
      label: t('changeRequests.decline_reason.jeopardized_main_deadline'),
      value: 'jeopardized_main_deadline',
    },
    {
      label: t('changeRequests.decline_reason.postponed_to_next_phase'),
      value: 'postponed_to_next_phase',
    },
    {
      label: t('changeRequests.decline_reason.rethinking_requirements'),
      value: 'rethinking_requirements',
    },
    {
      label: t('changeRequests.decline_reason.not_necessary_anymore'),
      value: 'not_necessary_anymore',
    },
    {
      label: t('changeRequests.decline_reason.will_solve_internally'),
      value: 'will_solve_internally',
    },
    {
      label: t('changeRequests.decline_reason.thought_it_was_included'),
      value: 'thought_it_was_included',
    },
  ];
};
