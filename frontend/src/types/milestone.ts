export type MilestoneStatus = 'open' | 'review' | 'approved' | 'released' | 'cancelled';

export interface Milestone {
  id: string;
  project_id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}

export interface MilestoneDto {
  project_id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
}

export interface ClientMilestone {
  id: string;
  name: string;
  description: string | null;
  status: MilestoneStatus;
  due_date: string | null;
  created_at: string;
  updated_at: string;
}
