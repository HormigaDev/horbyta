use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

fn validate_timezone(timezone: &str) -> Result<(), ValidationError> {
    if chrono_tz::Tz::from_str(timezone).is_err() {
        return Err(ValidationError::new("invalid_timezone"));
    }

    Ok(())
}

fn validate_password_confirmation(payload: &UpdateUserPasswordDto) -> Result<(), ValidationError> {
    if payload.new_password != payload.confirm_password {
        return Err(ValidationError::new("password_mismatch"));
    }

    if payload.current_password == payload.new_password {
        return Err(ValidationError::new("new_password_must_change"));
    }

    Ok(())
}

fn validate_non_empty_preferences_patch(payload: &UpdateUserPreferencesDto) -> Result<(), ValidationError> {
    if payload.theme.is_none()
        && payload.timezone.is_none()
        && payload.date_format.is_none()
        && payload.language.is_none()
        && payload.max_daily_hours.is_none()
    {
        return Err(ValidationError::new("empty_preferences_patch"));
    }

    Ok(())
}

fn validate_pin_task_payload(payload: &PinTaskDto) -> Result<(), ValidationError> {
    if payload.title.trim().is_empty() {
        return Err(ValidationError::new("title_required"));
    }
    Ok(())
}

fn validate_non_empty_notifications_patch(payload: &UpdateNotificationPreferencesDto) -> Result<(), ValidationError> {
    if payload.group_notifications.is_none()
        && payload.client_portal.is_none()
        && payload.billing.is_none()
        && payload.work.is_none()
    {
        return Err(ValidationError::new("empty_notifications_patch"));
    }

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserLanguagePreference {
    En,
    Es,
    Pt,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum UserDateFormatPreference {
    #[serde(rename = "DD/MM/YYYY")]
    DdMmYyyy,
    #[serde(rename = "MM/DD/YYYY")]
    MmDdYyyy,
    #[serde(rename = "YYYY/MM/DD")]
    YyyyMmDd,
}

impl Default for UserDateFormatPreference {
    fn default() -> Self {
        Self::DdMmYyyy
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ClientPortalNotifications {
    pub change_request_created: bool,
    pub change_request_updated: bool,
    pub change_request_approved: bool,
    pub change_request_declined: bool,
    pub change_request_cancelled: bool,
}

impl Default for ClientPortalNotifications {
    fn default() -> Self {
        Self {
            change_request_created: true,
            change_request_updated: true,
            change_request_approved: true,
            change_request_declined: true,
            change_request_cancelled: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct BillingNotifications {
    pub invoice_opened: bool,
    pub invoice_paid: bool,
    pub payment_received: bool,
}

impl Default for BillingNotifications {
    fn default() -> Self {
        Self {
            invoice_opened: true,
            invoice_paid: true,
            payment_received: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct WorkNotifications {
    pub milestone_released: bool,
}

impl Default for WorkNotifications {
    fn default() -> Self {
        Self {
            milestone_released: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct NotificationPreferences {
    pub group_notifications: bool,
    pub client_portal: ClientPortalNotifications,
    pub billing: BillingNotifications,
    pub work: WorkNotifications,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            group_notifications: false,
            client_portal: ClientPortalNotifications::default(),
            billing: BillingNotifications::default(),
            work: WorkNotifications::default(),
        }
    }
}

impl NotificationPreferences {
    pub fn is_enabled(&self, preference_key: &str) -> bool {
        match preference_key {
            "client_portal.change_request_created" => self.client_portal.change_request_created,
            "client_portal.change_request_updated" => self.client_portal.change_request_updated,
            "client_portal.change_request_approved" => self.client_portal.change_request_approved,
            "client_portal.change_request_declined" => self.client_portal.change_request_declined,
            "client_portal.change_request_cancelled" => self.client_portal.change_request_cancelled,
            "billing.invoice_opened" => self.billing.invoice_opened,
            "billing.invoice_paid" => self.billing.invoice_paid,
            "billing.payment_received" => self.billing.payment_received,
            "work.milestone_released" => self.work.milestone_released,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PinnedTask {
    pub id: String,
    pub record_number: i64,
    pub title: String,
    pub project_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UserPreferences {
    pub theme: UserThemePreference,
    pub timezone: String,
    pub date_format: UserDateFormatPreference,
    pub language: UserLanguagePreference,
    #[serde(default = "default_max_daily_hours")]
    pub max_daily_hours: i32,
    pub notifications: NotificationPreferences,
    #[serde(default)]
    pub pinned_tasks: Vec<PinnedTask>,
}

fn default_max_daily_hours() -> i32 {
    8
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: UserThemePreference::System,
            timezone: "UTC".to_string(),
            date_format: UserDateFormatPreference::DdMmYyyy,
            language: UserLanguagePreference::Es,
            max_daily_hours: default_max_daily_hours(),
            notifications: NotificationPreferences::default(),
            pinned_tasks: Vec::new(),
        }
    }
}

impl UserPreferences {
    pub fn from_json_value(raw: Option<&serde_json::Value>) -> Self {
        raw.and_then(|value| serde_json::from_value::<Self>(value.clone()).ok())
            .unwrap_or_default()
    }

    pub fn apply_general_patch(&mut self, patch: &UpdateUserPreferencesDto) {
        if let Some(theme) = patch.theme {
            self.theme = theme;
        }

        if let Some(timezone) = patch.timezone.as_ref() {
            self.timezone = timezone.clone();
        }

        if let Some(date_format) = patch.date_format {
            self.date_format = date_format;
        }

        if let Some(language) = patch.language {
            self.language = language;
        }

        if let Some(max_daily_hours) = patch.max_daily_hours {
            self.max_daily_hours = max_daily_hours;
        }
    }

    pub fn pin_task(&mut self, task: PinnedTask) {
        self.pinned_tasks.retain(|t| t.id != task.id);
        self.pinned_tasks.insert(0, task);
        self.pinned_tasks.truncate(10);
    }

    pub fn unpin_task(&mut self, task_id: &str) {
        self.pinned_tasks.retain(|t| t.id != task_id);
    }

    pub fn apply_notifications_patch(&mut self, patch: UpdateNotificationPreferencesDto) {
        if let Some(value) = patch.group_notifications {
            self.notifications.group_notifications = value;
        }

        if let Some(group) = patch.client_portal {
            if let Some(value) = group.change_request_created {
                self.notifications.client_portal.change_request_created = value;
            }
            if let Some(value) = group.change_request_updated {
                self.notifications.client_portal.change_request_updated = value;
            }
            if let Some(value) = group.change_request_approved {
                self.notifications.client_portal.change_request_approved = value;
            }
            if let Some(value) = group.change_request_declined {
                self.notifications.client_portal.change_request_declined = value;
            }
            if let Some(value) = group.change_request_cancelled {
                self.notifications.client_portal.change_request_cancelled = value;
            }
        }

        if let Some(group) = patch.billing {
            if let Some(value) = group.invoice_opened {
                self.notifications.billing.invoice_opened = value;
            }
            if let Some(value) = group.invoice_paid {
                self.notifications.billing.invoice_paid = value;
            }
            if let Some(value) = group.payment_received {
                self.notifications.billing.payment_received = value;
            }
        }

        if let Some(group) = patch.work {
            if let Some(value) = group.milestone_released {
                self.notifications.work.milestone_released = value;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateUserProfileDto {
    #[validate(length(min = 2, max = 100, message = "name must be between 2 and 100 characters"))]
    pub name: String,

    #[validate(email(message = "email must be a valid email address"))]
    #[validate(length(max = 255, message = "email must be at most 255 characters"))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_password_confirmation"))]
pub struct UpdateUserPasswordDto {
    #[validate(length(
        min = 1,
        max = 128,
        message = "current password must be between 1 and 128 characters"
    ))]
    pub current_password: String,

    #[validate(length(min = 12, max = 128, message = "new password must be between 12 and 128 characters"))]
    pub new_password: String,

    #[validate(length(
        min = 12,
        max = 128,
        message = "confirm password must be between 12 and 128 characters"
    ))]
    pub confirm_password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_non_empty_preferences_patch"))]
pub struct UpdateUserPreferencesDto {
    pub theme: Option<UserThemePreference>,

    #[validate(length(min = 1, max = 64, message = "timezone must be between 1 and 64 characters"))]
    #[validate(custom(function = "validate_timezone"))]
    pub timezone: Option<String>,

    pub date_format: Option<UserDateFormatPreference>,

    pub language: Option<UserLanguagePreference>,

    #[validate(range(min = 1, max = 14, message = "max daily hours must be between 1 and 14"))]
    pub max_daily_hours: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ClientPortalNotificationsPatch {
    pub change_request_created: Option<bool>,
    pub change_request_updated: Option<bool>,
    pub change_request_approved: Option<bool>,
    pub change_request_declined: Option<bool>,
    pub change_request_cancelled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BillingNotificationsPatch {
    pub invoice_opened: Option<bool>,
    pub invoice_paid: Option<bool>,
    pub payment_received: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(default, deny_unknown_fields)]
pub struct WorkNotificationsPatch {
    pub milestone_released: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_non_empty_notifications_patch"))]
pub struct UpdateNotificationPreferencesDto {
    pub group_notifications: Option<bool>,

    #[validate(nested)]
    pub client_portal: Option<ClientPortalNotificationsPatch>,

    #[validate(nested)]
    pub billing: Option<BillingNotificationsPatch>,

    #[validate(nested)]
    pub work: Option<WorkNotificationsPatch>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_pin_task_payload"))]
pub struct PinTaskDto {
    #[validate(length(min = 1, max = 255))]
    pub id: String,
    pub record_number: i64,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub project_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSettingsDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub preferences: UserPreferences,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl From<crate::entities::users::Model> for UserSettingsDto {
    fn from(model: crate::entities::users::Model) -> Self {
        let preferences = UserPreferences::from_json_value(model.preferences.as_ref());
        Self {
            id: model.id,
            name: model.name,
            email: model.email,
            preferences,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
