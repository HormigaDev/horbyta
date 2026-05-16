export type UiStateScope = 'datatable' | 'board' | 'pinned' | 'notification_log' | 'form_pref';

export interface UiStateEntryDto<T = unknown> {
  scope: string;
  key: string;
  value: T;
  updated_at: string;
}

export interface UiStateListResponse<T = unknown> {
  items: UiStateEntryDto<T>[];
}
