// Traducción de los errores del backend. Cada respuesta trae `code` (catálogo en
// src/error_codes.rs) y `params`, que reemplazan `$1`, `$2`… de `backendErrors.<code>`.

export interface ApiErrorTranslator {
  t: (key: string) => string;
  /** Si existe la clave en el idioma activo (los catálogos usan claves planas con puntos). */
  has: (key: string) => boolean;
}

export type ApiErrorKind = 'validation' | 'conflict' | 'forbidden' | 'notFound' | 'auth' | 'server';

/** Tipo de error para el título del diálogo, a partir del código HTTP. */
export const apiErrorKind = (status: number | null | undefined): ApiErrorKind => {
  if (status === 400 || status === 422) return 'validation';
  if (status === 409 || status === 423) return 'conflict';
  if (status === 403) return 'forbidden';
  if (status === 404) return 'notFound';
  if (status === 401) return 'auth';
  return 'server';
};

/** Nombre legible del campo de un error de validación (`billing.hourly_rate` → «tarifa por hora»). */
const fieldLabel = (i18n: ApiErrorTranslator, path: string): string => {
  const last =
    path
      .replace(/\[\d+\]/g, '')
      .split('.')
      .pop() ?? path;
  const key = `backendFields.${last}`;
  return i18n.has(key) ? i18n.t(key) : last.replace(/_/g, ' ');
};

/** Algunos parámetros son valores del sistema que también se traducen. */
const PARAM_LABELS: Record<string, (index: number) => string | null> = {
  TASK_STATUS_CATEGORY_REQUIRED: (index) => (index === 0 ? 'taskStatuses.category' : null),
  TASK_STATUS_CATEGORY_MISSING: (index) => (index === 0 ? 'taskStatuses.category' : null),
  INVOICE_STATUS_LOCKED: (index) => (index === 0 ? 'invoices.status' : null),
  INVOICE_STATUS_TRANSITION_INVALID: () => 'invoices.status',
};

const localizeParam = (
  i18n: ApiErrorTranslator,
  code: string,
  index: number,
  value: string,
): string => {
  if (index === 0 && /^(VALIDATION_|PASSWORD_)/.test(code)) return fieldLabel(i18n, value);
  const prefix = PARAM_LABELS[code]?.(index);
  const key = prefix ? `${prefix}.${value}` : null;
  return key && i18n.has(key) ? i18n.t(key) : value;
};

/** Sustituye `$n` por el parámetro n (de mayor a menor para que `$1` no pise `$10`). */
export const fillParams = (template: string, params: string[]): string =>
  params.reduceRight(
    (message, value, index) => message.split(`$${index + 1}`).join(value),
    template,
  );

/** Mensaje traducido del error, o null si el código no está en el catálogo. */
export const translateApiError = (
  i18n: ApiErrorTranslator,
  code: string | null | undefined,
  params: string[] = [],
): string | null => {
  if (!code) return null;
  const key = `backendErrors.${code}`;
  if (!i18n.has(key)) return null;
  return fillParams(
    i18n.t(key),
    params.map((value, index) => localizeParam(i18n, code, index, value)),
  );
};
