export interface ActivitySummaryDto {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface ActivityDetailDto {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface CreateActivityDto {
  name: string;
}

export interface UpdateActivityDto {
  name: string;
}
