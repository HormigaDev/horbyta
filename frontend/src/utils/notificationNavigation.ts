import type { AppNotification } from 'src/types/ui';
const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
export function scopeNotificationTarget(payload: unknown) {
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) return null;
  const p = payload as Record<string, unknown>;
  // Las notificaciones anteriores solo incluían proyecto y revisión.
  if (
    typeof p.project_id !== 'string' ||
    !uuid.test(p.project_id) ||
    !Number.isInteger(p.revision) ||
    Number(p.revision) < 0
  )
    return null;
  return {
    project: p.project_id,
    change:
      typeof p.change_request_id === 'string' && uuid.test(p.change_request_id)
        ? p.change_request_id
        : null,
    revision: Number(p.revision),
    message: typeof p.message_id === 'string' && uuid.test(p.message_id) ? p.message_id : null,
  };
}
export function notificationPreview(notice: Pick<AppNotification, 'payload' | 'body'>) {
  const text = notice.body?.replace(/\s+/g, ' ').trim() ?? '';
  const chars = Array.from(text);
  if (scopeNotificationTarget(notice.payload)) return chars.length <= 60 ? text : '';
  return chars.length <= 180 ? text : chars.slice(0, 177).join('') + '…';
}
