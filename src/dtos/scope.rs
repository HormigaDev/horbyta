use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Default, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ScopeDocument {
    #[serde(default)]
    #[validate(length(max = 20000))]
    pub deliverables: String,
    #[serde(default)]
    #[validate(length(max = 20000))]
    pub criteria: String,
    #[serde(default)]
    #[validate(length(max = 20000))]
    pub restrictions: String,
    #[serde(default)]
    #[validate(length(max = 20000))]
    pub exclusions: String,
    #[serde(default)]
    #[validate(length(max = 20000))]
    pub requirements: String,
    #[serde(default)]
    pub documents: Vec<ScopeContentDocument>,
    #[serde(default)]
    pub attachment_ids: Vec<Uuid>,
    #[serde(default)]
    pub reference_attachment_ids: Vec<Uuid>,
    #[serde(default)]
    pub based_on: Option<i32>,
    #[serde(default)]
    pub agreement_revision: Option<i32>,
    #[serde(default)]
    #[validate(length(max = 2000))]
    pub amendment_reason: String,
    #[serde(default)]
    pub precedence: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeContentDocument {
    pub id: Uuid,
    pub title: String,
    pub role: String,
    pub section: Option<String>,
    pub content: String,
    pub source_name: Option<String>,
    pub source_content: Option<String>,
    pub requirement_id: Option<String>,
}
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct PublishScopeDto {
    #[validate(range(min = 0))]
    pub expected_revision: i32,
    pub publish: bool,
    #[validate(nested)]
    pub document: ScopeDocument,
}
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ScopeDecisionDto {
    #[validate(range(min = 1))]
    pub revision: i32,
}
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ClassifyTaskDto {
    pub change_request_id: Option<Uuid>,
    #[validate(length(min = 1, max = 2000))]
    pub reason: String,
}
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ScopeCommentDto {
    #[validate(range(min = 1))]
    pub version: i32,
    #[validate(length(max = 4000))]
    pub text: String,
    #[serde(default)]
    #[validate(length(max = 10))]
    pub attachment_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ScopeFeedbackDto {
    #[validate(range(min = 1))]
    pub revision: i32,
    pub kind: String,
    #[validate(length(max = 4000))]
    pub text: String,
    pub document_id: Option<Uuid>,
    #[serde(default)]
    #[validate(length(max = 10))]
    pub attachment_ids: Vec<Uuid>,
}
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ScopePreparationDto {
    #[validate(length(min = 1, max = 2000))]
    pub reason: String,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ScopeRequirementDto {
    pub document_id: Uuid,
    #[validate(range(min = 1))]
    pub revision: i32,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ConversationWindowQuery {
    pub before: Option<Uuid>,
    pub after: Option<Uuid>,
    pub around: Option<Uuid>,
    pub notification: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct ConversationEvent {
    pub user_id: Uuid,
    pub client_id: Option<Uuid>,
    pub project_id: Uuid,
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ProjectMessageDto {
    #[validate(length(max = 4000))]
    pub text: String,
    #[serde(default)]
    #[validate(length(max = 10))]
    pub attachment_ids: Vec<Uuid>,
    pub scope_revision: Option<i32>,
    pub change_request_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    #[serde(default)]
    #[validate(length(max = 10))]
    pub mentions: Vec<ConversationMentionDto>,
}
#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConversationMentionDto {
    pub kind: String,
    pub id: Uuid,
    pub start: Option<usize>,
    pub end: Option<usize>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversationReadDto {
    pub message_id: Uuid,
}
#[derive(Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConversationChangeDto {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1, max = 20000))]
    pub description: String,
}
