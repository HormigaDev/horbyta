export type UserThemePreference = 'system' | 'light' | 'dark';
export type UserLanguagePreference = 'en' | 'es' | 'pt';
export type UserDateFormatPreference = 'DD/MM/YYYY' | 'MM/DD/YYYY' | 'YYYY/MM/DD';

export interface ClientsProjectsNotifications {
  proposal_decision: boolean;
  deliverable_approved: boolean;
  comment_or_review: boolean;
  added_to_new_project: boolean;
}

export interface DeliveriesDeadlinesNotifications {
  deadline_24h_48h: boolean;
  deliverable_overdue: boolean;
  milestone_completed: boolean;
}

export interface ContractsNotifications {
  contract_sent: boolean;
  contract_signed: boolean;
  contract_expiring: boolean;
}

export interface BillingNotifications {
  invoice_sent: boolean;
  invoice_overdue_unpaid: boolean;
  payment_received: boolean;
  auto_payment_reminder: boolean;
}

export interface TimeTrackingNotifications {
  long_running_timer: boolean;
  weekly_hours_summary: boolean;
}

export interface NotificationPreferences {
  clients_projects: ClientsProjectsNotifications;
  deliveries_deadlines: DeliveriesDeadlinesNotifications;
  contracts: ContractsNotifications;
  billing: BillingNotifications;
  time_tracking: TimeTrackingNotifications;
}

export interface UserPreferences {
  theme: UserThemePreference;
  timezone: string;
  date_format: UserDateFormatPreference;
  language: UserLanguagePreference;
  ai: {
    model: string;
    api_key_configured: boolean;
  };
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
  ai_model?: string;
  ai_api_key?: string;
}

export interface ClientsProjectsNotificationsPatch {
  proposal_decision?: boolean;
  deliverable_approved?: boolean;
  comment_or_review?: boolean;
  added_to_new_project?: boolean;
}

export interface DeliveriesDeadlinesNotificationsPatch {
  deadline_24h_48h?: boolean;
  deliverable_overdue?: boolean;
  milestone_completed?: boolean;
}

export interface ContractsNotificationsPatch {
  contract_sent?: boolean;
  contract_signed?: boolean;
  contract_expiring?: boolean;
}

export interface BillingNotificationsPatch {
  invoice_sent?: boolean;
  invoice_overdue_unpaid?: boolean;
  payment_received?: boolean;
  auto_payment_reminder?: boolean;
}

export interface TimeTrackingNotificationsPatch {
  long_running_timer?: boolean;
  weekly_hours_summary?: boolean;
}

export interface UpdateNotificationPreferencesDto {
  clients_projects?: ClientsProjectsNotificationsPatch;
  deliveries_deadlines?: DeliveriesDeadlinesNotificationsPatch;
  contracts?: ContractsNotificationsPatch;
  billing?: BillingNotificationsPatch;
  time_tracking?: TimeTrackingNotificationsPatch;
}
