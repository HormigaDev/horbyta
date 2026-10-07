export type ThemeIdentity = 'orbit' | 'linen' | 'forest';
export type InterfaceDensity = 'comfortable' | 'compact';
export interface AppearancePreference {
  theme_id: ThemeIdentity;
  density: InterfaceDensity;
  reduced_motion: boolean;
}
export type UserThemePreference = 'system' | 'light' | 'dark';
export type UserLanguagePreference = 'en' | 'es' | 'pt';
export type UserDateFormatPreference = 'DD/MM/YYYY' | 'MM/DD/YYYY' | 'YYYY/MM/DD';
export type UserRole = 'user' | 'admin';
export type UserStatus = 'active' | 'inactive' | 'blocked';

export interface UserNumberFormatPreference {
  thousand_separator: ',' | '.';
  decimal_separator: ',' | '.';
  decimal_quantity: number;
}

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

export interface PinnedTask {
  id: string;
  record_number: number;
  title: string;
  project_name: string | null;
}

export interface UserPreferences {
  appearance?: AppearancePreference;
  theme: UserThemePreference;
  timezone: string;
  date_format: UserDateFormatPreference;
  number_format: UserNumberFormatPreference;
  language: UserLanguagePreference;
  max_daily_hours: number;
  notifications: NotificationPreferences;
}

export interface User {
  id: string;
  name: string;
  email: string;
  role: UserRole;
  status: UserStatus;
  created_at: string;
  updated_at: string;
}

export interface UserSettings extends User {
  two_factor_auth_enabled: boolean;
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
  two_factor_auth_code?: string;
  two_factor_auth_backup_code?: string;
}

export interface UpdateUserPreferencesDto {
  appearance?: Partial<AppearancePreference>;
  theme?: UserThemePreference;
  timezone?: string;
  date_format?: UserDateFormatPreference;
  number_format?: UserNumberFormatPreference;
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

export interface AdminCreateUserDto {
  name: string;
  email: string;
  password: string;
  role: UserRole;
  status: UserStatus;
}

export interface AdminUpdateUserDto {
  name?: string;
  email?: string;
  password?: string;
  role?: UserRole;
  status?: UserStatus;
}

export interface AdminUserSummary {
  id: string;
  name: string;
  email: string;
  role: UserRole;
  status: UserStatus;
  created_at: string;
  updated_at: string;
}

export interface TotpSetup {
  otpauth_url: string;
  qr_png_base64: string;
  qr_data_uri: string;
  manual_entry_key: string;
}

export interface TotpStatus {
  enabled: boolean;
  setup_pending: boolean;
  backup_codes_remaining: number;
}

export interface TotpEnableResult {
  status: TotpStatus;
  backup_codes: string[];
  backup_codes_file: string;
}

export interface VerifyTotpCodeDto {
  code: string;
}

export interface DisableTotpDto {
  password: string;
  two_factor_auth_code?: string;
  two_factor_auth_backup_code?: string;
}
