import type { AttachmentSummaryDto } from 'src/types/attachment';

export const MAX_ATTACHMENT_FILE_BYTES = 50 * 1024 * 1024;
export const MAX_ATTACHMENT_TOTAL_BYTES = 50 * 1024 * 1024;

const IMAGE_EXTENSIONS = new Set(['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg']);
const ZIP_EXTENSIONS = new Set(['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz']);
const CODE_EXTENSIONS = new Set([
  'js',
  'jsx',
  'ts',
  'tsx',
  'vue',
  'rs',
  'py',
  'go',
  'java',
  'kt',
  'c',
  'h',
  'hpp',
  'cpp',
  'cs',
  'php',
  'rb',
  'swift',
  'sql',
  'json',
  'yml',
  'yaml',
  'toml',
  'md',
  'sh',
]);

const CODE_MIME_HINTS = [
  'text/',
  'application/json',
  'application/javascript',
  'application/x-javascript',
  'application/xml',
  'application/x-yaml',
  'application/x-toml',
] as const;

export type QFileModelValue = File | readonly File[] | null;

const isReadonlyFileArray = (value: QFileModelValue): value is readonly File[] => {
  return Array.isArray(value);
};

export const toFileArray = (value: QFileModelValue): File[] => {
  if (!value) return [];

  if (isReadonlyFileArray(value)) {
    return [...value];
  }

  return [value];
};

export const sumFileSizes = (files: readonly Pick<File, 'size'>[]): number => {
  return files.reduce((sum, file) => sum + file.size, 0);
};

export const formatStorageBytes = (bytes: number): string => {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'] as const;
  let value = bytes;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  const precision = value >= 10 || unitIndex === 0 ? 0 : 1;
  return `${value.toFixed(precision)} ${units[unitIndex]}`;
};

const getExtension = (fileName: string): string => {
  const cleanName = fileName.trim().toLowerCase();
  const dotIndex = cleanName.lastIndexOf('.');

  if (dotIndex < 0 || dotIndex === cleanName.length - 1) {
    return '';
  }

  return cleanName.slice(dotIndex + 1);
};

const isCodeMime = (mimeType: string): boolean => {
  const normalizedMime = mimeType.trim().toLowerCase();
  return CODE_MIME_HINTS.some((hint) => normalizedMime.startsWith(hint));
};

export const resolveAttachmentIcon = (
  attachment: Pick<AttachmentSummaryDto, 'mime_type' | 'original_name'>,
): string => {
  const normalizedMime = attachment.mime_type.trim().toLowerCase();
  const extension = getExtension(attachment.original_name);

  if (normalizedMime.startsWith('image/') || IMAGE_EXTENSIONS.has(extension)) {
    return 'image';
  }

  if (
    normalizedMime.includes('zip') ||
    normalizedMime.includes('compressed') ||
    ZIP_EXTENSIONS.has(extension)
  ) {
    return 'folder_zip';
  }

  if (isCodeMime(normalizedMime) || CODE_EXTENSIONS.has(extension)) {
    return 'code';
  }

  return 'insert_drive_file';
};

export const resolveFileIcon = (file: Pick<File, 'name' | 'type'>): string => {
  return resolveAttachmentIcon({
    original_name: file.name,
    mime_type: file.type || 'application/octet-stream',
  });
};
