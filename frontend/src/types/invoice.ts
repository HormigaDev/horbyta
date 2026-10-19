export type InvoiceStatus = 'draft' | 'open' | 'paid' | 'void' | 'uncollectible';
export type InvoiceProvider =
  | 'manual'
  | 'paypal'
  | 'stripe'
  | 'wise'
  | 'mercadopago'
  | 'revolut'
  | 'payoneer';

export interface InvoiceSummaryDto {
  id: string;
  record_number: bigint;
  project_id: string;
  project_name: string | null;
  client_name: string | null;
  amount: string;
  currency_type: string;
  status: InvoiceStatus;
  due_date: string | null;
  issued_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface InvoiceTimeEntryDto {
  id: string;
  task_id: string;
  duration_minutes: number;
  work_date: string;
  billable: boolean;
  applied_rate: string | null;
  notes: string | null;
  activity_name: string | null;
}

export interface InvoiceTaskGroupDto {
  task_id: string;
  task_record_number: bigint;
  task_title: string;
  total_duration_minutes: number;
  time_entries: InvoiceTimeEntryDto[];
}

export interface InvoiceDetailDto {
  id: string;
  record_number: bigint;
  user_id: string;
  project_id: string;
  project_name: string;
  project_status: string;
  client_id: string | null;
  client_name: string | null;
  client_email: string | null;
  amount: string;
  currency_type: string;
  status: InvoiceStatus;
  due_date: string | null;
  issued_at: string | null;
  provider: InvoiceProvider;
  provider_external_id: string | null;
  payment_link: string | null;
  notes: string | null;
  freelancer_notes: string | null;
  task_groups: InvoiceTaskGroupDto[];
  created_at: string;
  updated_at: string;
}

export interface CreateInvoiceDto {
  project_id: string;
  amount: string;
  due_date?: string | null;
  freelancer_notes?: string | null;
  time_entry_ids: string[];
  provider: InvoiceProvider;
  provider_external_id?: string | null;
  payment_link?: string | null;
}

export interface UpdateInvoiceDto {
  amount: string;
  due_date?: string | null;
  freelancer_notes?: string | null;
  time_entry_ids: string[];
  provider: InvoiceProvider;
  provider_external_id?: string | null;
  payment_link?: string | null;
}

export interface UpdateInvoiceStatusDto {
  status: InvoiceStatus;
}

export interface BillableTimeEntryDto {
  id: string;
  task_id: string;
  duration_minutes: number;
  work_date: string;
  billable: boolean;
  applied_rate: string | null;
  notes: string | null;
  activity_name: string | null;
}

export interface BillableTaskDto {
  task_id: string;
  task_record_number: bigint;
  task_title: string;
  total_duration_minutes: number;
  time_entries: BillableTimeEntryDto[];
}

export interface BillableTasksResponse {
  tasks: BillableTaskDto[];
  total_tasks: number;
  page: number;
  per_page: number;
}
