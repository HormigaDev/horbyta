export type ChangeRequestStatus =
  | 'draft'
  | 'submitted'
  | 'assessed'
  | 'approved'
  | 'declined'
  | 'implemented'
  | 'cancelled';

export type ChangeRequestOrigin = 'freelancer_dashboard' | 'client_portal';

export interface ChangeRequestSummaryDto {
  id: string;
  record_number: bigint;
  project_id: string;
  project_name: string | null;
  client_id: string;
  client_name: string | null;
  task_id: string | null;
  title: string;
  origin: ChangeRequestOrigin;
  status: ChangeRequestStatus;
  created_at: string;
  updated_at: string;
}

export interface ChangeRequestDetailDto {
  id: string;
  record_number: bigint;
  user_id: string;
  project_id: string;
  project_name: string | null;
  client_id: string;
  client_name: string | null;
  task_id: string | null;
  title: string;
  client_request_description: string;
  technical_assessment: string | null;
  impact_hours: string;
  impact_cost: string;
  schedule_impact_days: number;
  origin: ChangeRequestOrigin;
  status: ChangeRequestStatus;
  decline_reason: string;
  client_approved_at: string | null;
  client_signature_ip: string | null;
  created_at: string;
  updated_at: string;
  version: number;
}

export interface ClientChangeRequestSummaryDto {
  id: string;
  record_number: bigint;
  project_id: string;
  project_name: string | null;
  task_id: string | null;
  title: string;
  status: ChangeRequestStatus;
  created_at: string;
  updated_at: string;
}

export interface ClientChangeRequestDetailDto {
  id: string;
  record_number: bigint;
  project_id: string;
  project_name: string | null;
  task_id: string | null;
  title: string;
  client_request_description: string;
  technical_assessment: string | null;
  impact_hours: string;
  impact_cost: string;
  schedule_impact_days: number;
  origin: ChangeRequestOrigin;
  status: ChangeRequestStatus;
  client_approved_at: string | null;
  client_signature_ip: string | null;
  created_at: string;
  updated_at: string;
  version: number;
}

export interface CreateChangeRequestDto {
  project_id: string;
  client_id: string;
  task_id?: string | null;
  title: string;
  client_request_description: string;
}

export interface UpdateChangeRequestDto {
  project_id: string;
  client_id: string;
  task_id?: string | null;
  title: string;
  client_request_description: string;
  technical_assessment?: string | null;
  impact_hours: string;
  impact_cost: string;
  schedule_impact_days: number;
  version: number;
}

export interface AssessChangeRequestDto {
  technical_assessment: string;
  impact_hours: string;
  impact_cost: string;
  schedule_impact_days: number;
}

export interface AssignChangeRequestTaskDto {
  task_id?: string | null;
  version: number;
}

export interface ClientCreateChangeRequestDto {
  project_id: string;
  title: string;
  client_request_description: string;
}

export interface ClientUpdateChangeRequestDto {
  title: string;
  client_request_description: string;
  version: number;
}

export interface ClientCancelChangeRequestDto {
  version: number;
}

export interface ClientApproveChangeRequestDto {
  client_signature_ip?: string | null;
}
