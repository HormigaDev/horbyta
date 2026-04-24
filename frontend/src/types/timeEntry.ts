import type { PaginatedResponse } from 'src/types/query';

export interface TimeEntrySummaryDto {
  id: string;
  task_id: string;
  activity: string;
  invoice_id: string | null;
  start_time: string | null;
  end_time: string | null;
  duration_minutes: number;
  work_date: string;
  billable: boolean;
  applied_rate: string | null;
  notes: string | null;
  task_title: string | null;
  activity_name: string | null;
  created_at: string;
  updated_at: string;
}

export interface TimeEntryDetailDto {
  id: string;
  user_id: string;
  task_id: string;
  activity: string;
  invoice_id: string | null;
  start_time: string | null;
  end_time: string | null;
  duration_minutes: number;
  work_date: string;
  billable: boolean;
  applied_rate: string | null;
  notes: string | null;
  task_title: string | null;
  activity_name: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTimeEntryDto {
  task_id: string;
  activity: string;
  billable: boolean;
  duration_minutes: number;
  work_date?: string;
  notes?: string;
}

export interface StartTimeEntryTimerDto {
  task_id: string;
  activity: string;
  billable: boolean;
  start_time?: string;
  work_date?: string;
  notes?: string;
}

export interface UpdateTimeEntryDto {
  task_id: string;
  activity: string;
  billable: boolean;
  duration_minutes: number;
  work_date: string;
  notes?: string;
}

export interface StopTimeEntryTimerDto {
  end_time?: string;
}

export interface ActiveTimeEntryTimerResponse {
  item: TimeEntryDetailDto | null;
}

export interface TimeEntrySearchResponse extends PaginatedResponse<TimeEntrySummaryDto> {
  daily_totals_by_date: Record<string, number>;
  daily_limit_minutes: number;
}
