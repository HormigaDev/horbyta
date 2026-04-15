export type MilestoneStatus = 'open' | 'review' | 'approved' | 'released' | 'cancelled';

export interface MilestoneSummaryDto {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface MilestoneDetailDto {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateMilestoneDto {
  project_id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
}

export interface UpdateMilestoneDto {
  project_id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
}

export interface ClientMilestoneSummaryDto {
  id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface ClientMilestoneDetailDto {
  id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}
