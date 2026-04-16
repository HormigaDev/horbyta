export interface AttachmentSummaryDto {
  id: string;
  project_id: string | null;
  task_id: string | null;
  original_name: string;
  mime_type: string;
  size_bytes: number;
  created_at: string;
  updated_at: string;
}

export interface AttachmentDetailDto {
  id: string;
  user_id: string;
  project_id: string | null;
  task_id: string | null;
  original_name: string;
  storage_key: string;
  mime_type: string;
  size_bytes: number;
  created_at: string;
  updated_at: string;
}

export interface AttachmentUploadResponseDto {
  items: AttachmentDetailDto[];
  uploaded_bytes: number;
  uploaded_count: number;
}

export interface AttachmentStorageUsageDto {
  used_bytes: number;
  limit_bytes: number;
  remaining_bytes: number;
}

export interface AttachmentListItem {
  id: string;
  name: string;
  sizeLabel: string;
  icon: string;
}
