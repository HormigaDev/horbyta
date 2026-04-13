use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::constants::ALLOWED_AI_MODELS;

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

fn validate_ai_model(model: &str) -> Result<(), ValidationError> {
    if ALLOWED_AI_MODELS.contains(&model) {
        return Ok(());
    }

    Err(ValidationError::new("invalid_ai_model"))
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
        && payload.language.is_none()
        && payload.ai_model.is_none()
        && payload.ai_api_key.is_none()
    {
        return Err(ValidationError::new("empty_preferences_patch"));
    }

    Ok(())
}

fn validate_non_empty_notifications_patch(payload: &UpdateNotificationPreferencesDto) -> Result<(), ValidationError> {
    if payload.clients_projects.is_none()
        && payload.deliveries_deadlines.is_none()
        && payload.contracts.is_none()
        && payload.billing.is_none()
        && payload.time_tracking.is_none()
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UserAiPreferences {
    pub model: String,
    #[serde(default)]
    pub api_key_encrypted: Option<String>,
    #[serde(default)]
    pub api_key_configured: bool,
}

impl Default for UserAiPreferences {
    fn default() -> Self {
        Self {
            model: "google/gemini-2.5-flash".to_string(),
            api_key_encrypted: None,
            api_key_configured: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct ClientsProjectsNotifications {
    pub proposal_decision: bool,
    pub deliverable_approved: bool,
    pub comment_or_review: bool,
    pub added_to_new_project: bool,
}

impl Default for ClientsProjectsNotifications {
    fn default() -> Self {
        Self {
            proposal_decision: true,
            deliverable_approved: true,
            comment_or_review: true,
            added_to_new_project: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct DeliveriesDeadlinesNotifications {
    pub deadline_24h_48h: bool,
    pub deliverable_overdue: bool,
    pub milestone_completed: bool,
}

impl Default for DeliveriesDeadlinesNotifications {
    fn default() -> Self {
        Self {
            deadline_24h_48h: true,
            deliverable_overdue: true,
            milestone_completed: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct ContractsNotifications {
    pub contract_sent: bool,
    pub contract_signed: bool,
    pub contract_expiring: bool,
}

impl Default for ContractsNotifications {
    fn default() -> Self {
        Self {
            contract_sent: true,
            contract_signed: true,
            contract_expiring: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct BillingNotifications {
    pub invoice_sent: bool,
    pub invoice_overdue_unpaid: bool,
    pub payment_received: bool,
    pub auto_payment_reminder: bool,
}

impl Default for BillingNotifications {
    fn default() -> Self {
        Self {
            invoice_sent: true,
            invoice_overdue_unpaid: true,
            payment_received: true,
            auto_payment_reminder: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct TimeTrackingNotifications {
    pub long_running_timer: bool,
    pub weekly_hours_summary: bool,
}

impl Default for TimeTrackingNotifications {
    fn default() -> Self {
        Self {
            long_running_timer: true,
            weekly_hours_summary: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct NotificationPreferences {
    pub clients_projects: ClientsProjectsNotifications,
    pub deliveries_deadlines: DeliveriesDeadlinesNotifications,
    pub contracts: ContractsNotifications,
    pub billing: BillingNotifications,
    pub time_tracking: TimeTrackingNotifications,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct UserPreferences {
    pub theme: UserThemePreference,
    pub timezone: String,
    pub language: UserLanguagePreference,
    pub ai: UserAiPreferences,
    pub notifications: NotificationPreferences,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: UserThemePreference::System,
            timezone: "UTC".to_string(),
            language: UserLanguagePreference::Es,
            ai: UserAiPreferences::default(),
            notifications: NotificationPreferences::default(),
        }
    }
}

impl UserPreferences {
    pub fn from_json_value(raw: Option<&serde_json::Value>) -> Self {
        let mut preferences = raw
            .and_then(|value| serde_json::from_value::<Self>(value.clone()).ok())
            .unwrap_or_default();

        preferences.ai.api_key_configured = preferences
            .ai
            .api_key_encrypted
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false);

        preferences
    }

    pub fn apply_general_patch(&mut self, patch: &UpdateUserPreferencesDto) {
        if let Some(theme) = patch.theme {
            self.theme = theme;
        }

        if let Some(timezone) = patch.timezone.as_ref() {
            self.timezone = timezone.clone();
        }

        if let Some(language) = patch.language {
            self.language = language;
        }

        if let Some(model) = patch.ai_model.as_ref() {
            self.ai.model = model.clone();
        }
    }

    pub fn apply_notifications_patch(&mut self, patch: UpdateNotificationPreferencesDto) {
        if let Some(group) = patch.clients_projects {
            if let Some(value) = group.proposal_decision {
                self.notifications.clients_projects.proposal_decision = value;
            }
            if let Some(value) = group.deliverable_approved {
                self.notifications.clients_projects.deliverable_approved = value;
            }
            if let Some(value) = group.comment_or_review {
                self.notifications.clients_projects.comment_or_review = value;
            }
            if let Some(value) = group.added_to_new_project {
                self.notifications.clients_projects.added_to_new_project = value;
            }
        }

        if let Some(group) = patch.deliveries_deadlines {
            if let Some(value) = group.deadline_24h_48h {
                self.notifications.deliveries_deadlines.deadline_24h_48h = value;
            }
            if let Some(value) = group.deliverable_overdue {
                self.notifications.deliveries_deadlines.deliverable_overdue = value;
            }
            if let Some(value) = group.milestone_completed {
                self.notifications.deliveries_deadlines.milestone_completed = value;
            }
        }

        if let Some(group) = patch.contracts {
            if let Some(value) = group.contract_sent {
                self.notifications.contracts.contract_sent = value;
            }
            if let Some(value) = group.contract_signed {
                self.notifications.contracts.contract_signed = value;
            }
            if let Some(value) = group.contract_expiring {
                self.notifications.contracts.contract_expiring = value;
            }
        }

        if let Some(group) = patch.billing {
            if let Some(value) = group.invoice_sent {
                self.notifications.billing.invoice_sent = value;
            }
            if let Some(value) = group.invoice_overdue_unpaid {
                self.notifications.billing.invoice_overdue_unpaid = value;
            }
            if let Some(value) = group.payment_received {
                self.notifications.billing.payment_received = value;
            }
            if let Some(value) = group.auto_payment_reminder {
                self.notifications.billing.auto_payment_reminder = value;
            }
        }

        if let Some(group) = patch.time_tracking {
            if let Some(value) = group.long_running_timer {
                self.notifications.time_tracking.long_running_timer = value;
            }
            if let Some(value) = group.weekly_hours_summary {
                self.notifications.time_tracking.weekly_hours_summary = value;
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateUserProfileDto {
    #[validate(length(min = 2, max = 100, message = "El nombre debe tener entre 2 y 100 caracteres"))]
    pub name: String,

    #[validate(email(message = "Correo electrónico inválido"))]
    #[validate(length(max = 255, message = "El correo electrónico no puede superar los 255 caracteres"))]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_password_confirmation"))]
pub struct UpdateUserPasswordDto {
    #[validate(length(min = 1, max = 128, message = "La contraseña actual es obligatoria"))]
    pub current_password: String,

    #[validate(length(
        min = 8,
        max = 128,
        message = "La nueva contraseña debe tener entre 8 y 128 caracteres"
    ))]
    pub new_password: String,

    #[validate(length(min = 8, max = 128, message = "La confirmación debe tener entre 8 y 128 caracteres"))]
    pub confirm_password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_non_empty_preferences_patch"))]
pub struct UpdateUserPreferencesDto {
    pub theme: Option<UserThemePreference>,

    #[validate(length(min = 1, max = 64, message = "Timezone inválida"))]
    #[validate(custom(function = "validate_timezone"))]
    pub timezone: Option<String>,

    pub language: Option<UserLanguagePreference>,

    #[validate(length(min = 1, max = 128, message = "Modelo AI invalido"))]
    #[validate(custom(function = "validate_ai_model"))]
    pub ai_model: Option<String>,

    #[validate(length(min = 8, max = 4096, message = "API key AI invalida"))]
    pub ai_api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ClientsProjectsNotificationsPatch {
    pub proposal_decision: Option<bool>,
    pub deliverable_approved: Option<bool>,
    pub comment_or_review: Option<bool>,
    pub added_to_new_project: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(default, deny_unknown_fields)]
pub struct DeliveriesDeadlinesNotificationsPatch {
    pub deadline_24h_48h: Option<bool>,
    pub deliverable_overdue: Option<bool>,
    pub milestone_completed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ContractsNotificationsPatch {
    pub contract_sent: Option<bool>,
    pub contract_signed: Option<bool>,
    pub contract_expiring: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BillingNotificationsPatch {
    pub invoice_sent: Option<bool>,
    pub invoice_overdue_unpaid: Option<bool>,
    pub payment_received: Option<bool>,
    pub auto_payment_reminder: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(default, deny_unknown_fields)]
pub struct TimeTrackingNotificationsPatch {
    pub long_running_timer: Option<bool>,
    pub weekly_hours_summary: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
#[validate(schema(function = "validate_non_empty_notifications_patch"))]
pub struct UpdateNotificationPreferencesDto {
    #[validate(nested)]
    pub clients_projects: Option<ClientsProjectsNotificationsPatch>,

    #[validate(nested)]
    pub deliveries_deadlines: Option<DeliveriesDeadlinesNotificationsPatch>,

    #[validate(nested)]
    pub contracts: Option<ContractsNotificationsPatch>,

    #[validate(nested)]
    pub billing: Option<BillingNotificationsPatch>,

    #[validate(nested)]
    pub time_tracking: Option<TimeTrackingNotificationsPatch>,
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
        let mut preferences = UserPreferences::from_json_value(model.preferences.as_ref());
        // Nunca devolver la clave encriptada al frontend
        preferences.ai.api_key_encrypted = None;
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
