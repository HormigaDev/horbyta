import type { QueryPayload, PaginatedResponse } from '../types/query';
import type {
  PaymentSummaryDto,
  PaymentDetailDto,
  CreatePaymentDto,
  CreateRefundDto,
  UpdatePaymentDto,
} from '../types/payment';
import { apiFetch } from '../boot/fetch';

export const PaymentsService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<PaymentSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/payments/search?${params.toString()}`, { method: 'GET' });
  },

  getById: (id: string): Promise<PaymentDetailDto> =>
    apiFetch(`/payments/${id}`, { method: 'GET' }),

  create: (data: CreatePaymentDto): Promise<PaymentDetailDto> =>
    apiFetch('/payments', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  createRefund: (data: CreateRefundDto): Promise<PaymentDetailDto> =>
    apiFetch('/payments/refund', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdatePaymentDto): Promise<PaymentDetailDto> =>
    apiFetch(`/payments/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/payments/${id}`, { method: 'DELETE' }),
};
