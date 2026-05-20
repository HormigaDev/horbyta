use uuid::Uuid;

use crate::{
    dtos::notification::NotificationEvent, entities::notifications::Model as NotificationModel, state::AppState,
};

pub fn publish_notification(state: &AppState, user_id: Uuid, notification: Option<NotificationModel>) {
    if let Some(notification) = notification {
        let _ = state.notification_bus.send(NotificationEvent { user_id, notification });
    }
}
