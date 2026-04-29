export type UserThemePreference = 'system' | 'light' | 'dark';
export type UserLanguagePreference = 'en' | 'es' | 'pt';
export type UserDateFormatPreference = 'DD/MM/YYYY' | 'MM/DD/YYYY' | 'YYYY/MM/DD';

export interface ClientPortalNotifications {
  change_request_created: boolean;
  change_request_updated: boolean;
  change_request_approved: boolean;
  change_request_declined: boolean;
  change_request_cancelled: boolean;
}

export interface BillingNotifications {
  invoice_opened: boolean;
  invoice_paid: boolean;
  payment_received: boolean;
}

export interface WorkNotifications {
  milestone_released: boolean;
}

export interface NotificationPreferences {
  group_notifications: boolean;
  client_portal: ClientPortalNotifications;
  billing: BillingNotifications;
  work: WorkNotifications;
}

export interface UserPreferences {
  theme: UserThemePreference;
  timezone: string;
  date_format: UserDateFormatPreference;
  language: UserLanguagePreference;
  max_daily_hours: number;
  notifications: NotificationPreferences;
}

export interface User {
  id: string;
  name: string;
  email: string;
  created_at: string;
  updated_at: string;
}

export interface UserSettings extends User {
  preferences: UserPreferences;
}

export interface UpdateUserProfileDto {
  name: string;
  email: string;
}

export interface UpdateUserPasswordDto {
  current_password: string;
  new_password: string;
  confirm_password: string;
}

export interface UpdateUserPreferencesDto {
  theme?: UserThemePreference;
  timezone?: string;
  date_format?: UserDateFormatPreference;
  language?: UserLanguagePreference;
  max_daily_hours?: number;
}

export interface ClientPortalNotificationsPatch {
  change_request_created?: boolean;
  change_request_updated?: boolean;
  change_request_approved?: boolean;
  change_request_declined?: boolean;
  change_request_cancelled?: boolean;
}

export interface BillingNotificationsPatch {
  invoice_opened?: boolean;
  invoice_paid?: boolean;
  payment_received?: boolean;
}

export interface WorkNotificationsPatch {
  milestone_released?: boolean;
}

export interface UpdateNotificationPreferencesDto {
  group_notifications?: boolean;
  client_portal?: ClientPortalNotificationsPatch;
  billing?: BillingNotificationsPatch;
  work?: WorkNotificationsPatch;
}
