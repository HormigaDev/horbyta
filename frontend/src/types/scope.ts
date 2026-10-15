export interface ScopeDocument {
  deliverables: string;
  criteria: string;
  restrictions: string;
  exclusions: string;
  requirements: string;
  documents?: ScopeContentDocument[];
  attachment_ids?: string[];
  reference_attachment_ids?: string[];
  based_on?: number | null;
  agreement_revision?: number | null;
  amendment_reason?: string;
  precedence?: string;
}
export interface ScopeContentDocument {
  id: string;
  title: string;
  role: 'main' | 'annex' | 'reference';
  section: string | null;
  content: string;
  source_name: string | null;
  source_content: string | null;
  requirement_id: string | null;
  version?: number;
  hash?: string;
  source_hash?: string;
}
export interface ScopeRevision {
  revision: number;
  document: ScopeDocument;
  published: boolean;
  accepted_at: string | null;
  accepted_by: string | null;
  created_at: string;
  status:
    | 'draft'
    | 'pending'
    | 'superseded'
    | 'accepted'
    | 'consolidated'
    | 'rejected'
    | 'changes_requested';
  revision_kind: 'proposal' | 'consolidation';
  previous_accepted_revision: number | null;
  source_change_id: string | null;
  approved_changes: {
    id: string;
    title: string;
    offer_version: number;
    description: string;
    assessment: string | null;
    amount: string;
    currency_type: string | null;
    approved_at: string;
    accepted_ip: string | null;
    schedule_before: string | null;
    schedule_after: string | null;
  }[];
  bundle_hash: string | null;
  accepted_ip: string | null;
  accepted_ip_source: string | null;
  accepted_name: string | null;
  acceptance_statement: string | null;
  task_snapshot: { id: string; title: string }[];
  billing_snapshot: {
    currency_type: string;
    billing_type: string;
    fixed_price: string | null;
    hourly_rate: string | null;
    retainer_amount: string | null;
  } | null;
  milestone_snapshot: { id: string; name: string; due_date: string | null }[];
  attachment_snapshot: {
    id: string;
    original_name: string;
    scope_role?: 'annex' | 'reference';
    sha256?: string;
  }[];
}
export interface ScopeProject {
  project_id: string;
  client_id: string | null;
  last_revision: number;
  accepted_revision: number | null;
  published_revision: number | null;
  active: boolean;
  enforce_scope: boolean;
  current_billing: ScopeRevision['billing_snapshot'];
  current_work: { id: string; title: string }[];
  events: ScopeConversationMessage[];
  available_attachments: { id: string; original_name: string; size_bytes: number }[];
  revisions: ScopeRevision[];
  changes: { id: string; title: string; status: string; version: number; legacy: boolean }[];
  milestones: { id: string; name: string; due_date: string | null; status: string }[];
}
export interface ScopeChange {
  current: {
    source_message_id?: string | null;
    project_id: string;
    project_name: string;
    id: string;
    version: number;
    status: string;
    scope_revision: number | null;
    client_signature_ip: string | null;
    client_signature_ip_source: string | null;
    impact_cost: string;
    currency_type: string | null;
    billing_terms: string | null;
    schedule_before: string | null;
    schedule_after: string | null;
    legacy: boolean;
  };
  revisions: {
    version: number;
    recorded_at: string;
    provenance: string;
    snapshot: {
      title: string;
      client_request_description: string;
      technical_assessment: string | null;
      status: string;
      impact_hours: string;
      impact_cost: string;
      currency_type: string | null;
      schedule_impact_days: number;
      schedule_before: string | null;
      schedule_after: string | null;
      billing_terms: string | null;
    };
  }[];
  events: {
    id: string;
    kind: string;
    version: number;
    actor_kind: string;
    actor_name?: string | null;
    recorded_at: string;
    body: { text?: string };
  }[];
  effect: {
    invoice_id: string | null;
    invoice_number: number | null;
    invoice_status: string | null;
    amount: string;
    currency_type: string | null;
    paid_amount: string;
  } | null;
  tasks: { id: string; title: string; status: string; allowed: boolean }[];
}
export interface ScopeTask {
  task_id: string;
  project_id: string | null;
  controlled: boolean;
  active: boolean;
  preparation_reason: string | null;
  requirements: { document_id: string; revision: number }[];
  allowed: boolean;
  link: {
    baseline_revision: number | null;
    change_request_id: string | null;
    reason: string;
  } | null;
}
export interface ScopeNotice {
  id: string;
  kind: 'baseline' | 'change';
  record_id: string;
  revision: number;
  title: string;
  created_at: string;
}

export interface ScopeConversationMessage {
  id: string;
  revision: number;
  kind: string;
  text: string;
  document_id: string | null;
  actor_kind: string;
  actor_name?: string | null;
  context?: {
    scope_revision?: number;
    document_id?: string | null;
    change_request_id?: string;
    change_version?: number;
    change_title?: string;
    source_message_id?: string | null;
  };
  mentions?: ConversationResource[];
  recorded_at: string;
  attachments?: { id: string; original_name: string; mime_type: string; size_bytes: number }[];
}
export interface ScopeConversationWindow {
  focus_id?: string | null;
  items: ScopeConversationMessage[];
  has_older: boolean;
  has_newer: boolean;
}

export interface ConversationResource {
  kind: 'project' | 'milestone' | 'change' | 'file';
  id: string;
  label: string;
  start?: number;
  end?: number;
  mime_type?: string;
  size_bytes?: number;
}
