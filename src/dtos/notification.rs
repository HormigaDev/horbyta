use serde_json::Value;
use uuid::Uuid;

use crate::entities::notifications::Model as NotificationModel;

#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub user_id: Uuid,
    pub notification: NotificationModel,
}

pub struct CreateNotificationInput {
    pub user_id: Uuid,
    pub preference_key: &'static str,
    pub title: String,
    pub body: Option<String>,
    pub payload: Option<Value>,
    pub severity: &'static str,
    pub source: &'static str,
    pub entity_type: Option<&'static str>,
    pub entity_id: Option<Uuid>,
}
