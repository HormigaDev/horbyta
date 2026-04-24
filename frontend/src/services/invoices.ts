import type { QueryPayload, PaginatedResponse } from '../types/query';
import type {
  InvoiceSummaryDto,
  InvoiceDetailDto,
  CreateInvoiceDto,
  UpdateInvoiceDto,
  UpdateInvoiceStatusDto,
  BillableTasksResponse,
} from '../types/invoice';
import { apiFetch } from '../boot/fetch';

export const InvoicesService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<InvoiceSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/invoices/search?${params.toString()}`, { method: 'GET' });
  },

  getById: (id: string): Promise<InvoiceDetailDto> =>
    apiFetch(`/invoices/${id}`, { method: 'GET' }),

  create: (data: CreateInvoiceDto): Promise<InvoiceDetailDto> =>
    apiFetch('/invoices', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateInvoiceDto): Promise<InvoiceDetailDto> =>
    apiFetch(`/invoices/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  updateStatus: (id: string, data: UpdateInvoiceStatusDto): Promise<InvoiceDetailDto> =>
    apiFetch(`/invoices/${id}/status`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/invoices/${id}`, { method: 'DELETE' }),

  getBillableTasks: (
    projectId: string,
    page = 1,
    perPage = 100,
    invoiceId?: string,
  ): Promise<BillableTasksResponse> => {
    const params = new URLSearchParams({
      page: String(page),
      per_page: String(perPage),
    });
    if (invoiceId) {
      params.set('invoice_id', invoiceId);
    }
    return apiFetch(`/invoices/project/${projectId}/billable-tasks?${params.toString()}`, {
      method: 'GET',
    });
  },
};
