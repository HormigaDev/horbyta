export type PaymentTransactionType = 'charge' | 'refund';
export type PaymentStatus = 'completed' | 'partially_refunded' | 'fully_refunded' | 'refunded';

export interface PaymentInvoiceReferenceDto {
  id: string;
  record_number: bigint;
  status: string;
}

export interface RelatedRefundDto {
  id: string;
  record_number: bigint;
  amount: string;
  payment_date: string;
  notes: string | null;
}

export interface ParentPaymentReferenceDto {
  id: string;
  record_number: bigint;
  payment_date: string;
}

export interface PaymentSummaryDto {
  id: string;
  record_number: bigint;
  project_name: string | null;
  client_name: string | null;
  invoice_record_number: bigint | null;
  amount: string;
  currency_type: string;
  transaction_type: PaymentTransactionType;
  payment_date: string;
  payment_method: string | null;
  provider: string;
  created_at: string;
  updated_at: string;
}

export interface PaymentDetailDto {
  id: string;
  record_number: bigint;
  project_id: string;
  project_name: string;
  client_id: string | null;
  client_name: string | null;
  invoice: PaymentInvoiceReferenceDto | null;
  amount: string;
  currency_type: string;
  transaction_type: PaymentTransactionType;
  status: PaymentStatus;
  related_refunds: RelatedRefundDto[];
  parent_payment: ParentPaymentReferenceDto | null;
  payment_date: string;
  payment_method: string | null;
  reference_code: string | null;
  provider: string;
  provider_external_id: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreatePaymentDto {
  project_id: string;
  invoice_id?: string | null;
  amount: string;
  payment_date?: string | null;
  payment_method?: string | null;
  reference_code?: string | null;
  notes?: string | null;
}

export interface CreateRefundDto {
  parent_payment_id: string;
  amount: string;
  refund_date?: string | null;
  notes?: string | null;
}

export interface UpdatePaymentDto {
  payment_method?: string | null;
  reference_code?: string | null;
  notes?: string | null;
}
