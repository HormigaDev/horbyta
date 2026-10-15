export interface PreviewAttachment {
  id: string;
  original_name?: string;
  mime_type?: string;
}
export type PreviewKind = 'markdown' | 'text' | 'image' | 'audio' | 'video' | 'pdf' | 'unsupported';
const metadata = new Map<string, PreviewAttachment>();
export function registerPreviewAttachments(items: PreviewAttachment[]) {
  for (const item of items) metadata.set(item.id, { ...metadata.get(item.id), ...item });
}
export function getPreviewAttachment(id: string): PreviewAttachment {
  return metadata.get(id) ?? { id };
}
export function previewKind(name = '', mime = ''): PreviewKind {
  const type = mime.split(';')[0]?.trim().toLowerCase() ?? '';
  if (
    /\.(docx?|xlsx?|pptx?|od[tpbs]|zip|rar|7z|gz|bz2|xz|exe|dll|bin|wasm|woff2?|ttf|otf)$/i.test(
      name,
    )
  )
    return 'unsupported';
  if (type === 'application/pdf') return 'pdf';
  if (type.startsWith('image/')) return 'image';
  if (type.startsWith('audio/')) return 'audio';
  if (type.startsWith('video/')) return 'video';
  if (['text/markdown', 'text/x-markdown'].includes(type)) return 'markdown';
  // Las extensiones solo suplen MIME genéricos; HTML nunca se ejecuta como documento.
  if (type.startsWith('text/') && type !== 'text/plain') return 'text';
  if (
    [
      'application/json',
      'application/xml',
      'application/javascript',
      'application/x-javascript',
      'application/yaml',
      'application/x-yaml',
      'application/toml',
      'application/sql',
    ].includes(type) ||
    /^application\/[\w.+-]+\+(json|xml)$/.test(type)
  )
    return 'text';
  if (type && !['application/octet-stream', 'text/plain'].includes(type)) return 'unsupported';
  if (/\.(md|markdown)$/i.test(name)) return 'markdown';
  if (/\.pdf$/i.test(name)) return 'pdf';
  if (/\.(png|jpe?g|gif|webp|avif|apng|bmp|svg|ico)$/i.test(name)) return 'image';
  if (/\.(mp3|wav|ogg|oga|m4a|aac|flac|opus)$/i.test(name)) return 'audio';
  if (/\.(mp4|webm|ogv|mov|m4v)$/i.test(name)) return 'video';
  return 'text';
}
export function decodePreviewText(bytes: ArrayBuffer, mime = ''): string | null {
  const header = new Uint8Array(bytes, 0, Math.min(3, bytes.byteLength));
  const charset = /charset=["']?([^;"'\s]+)/i.exec(mime)?.[1];
  const encoding =
    header[0] === 0xff && header[1] === 0xfe
      ? 'utf-16le'
      : header[0] === 0xfe && header[1] === 0xff
        ? 'utf-16be'
        : (charset ?? 'utf-8');
  try {
    const text = new TextDecoder(encoding, { fatal: true }).decode(bytes);
    return Array.from(text).some((character) => {
      const code = character.charCodeAt(0);
      return code < 32 && ![9, 10, 12, 13].includes(code);
    })
      ? null
      : text;
  } catch {
    return null;
  }
}
export function attachmentIdFromUrl(value: string): string | null {
  let url: URL;
  try {
    url = new URL(value, window.location.origin);
  } catch {
    return null;
  }
  if (url.origin !== window.location.origin) return null;
  return /^\/api\/v1\/attachments\/([0-9a-f-]{36})\/raw\/?$/i.exec(url.pathname)?.[1] ?? null;
}
