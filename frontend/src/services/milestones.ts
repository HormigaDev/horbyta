import { apiFetch } from 'src/boot/fetch';
import { buildMultipartPayload } from 'src/services/attachments';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { MarkdownPendingFile } from 'src/types/markdown';
import type {
  CreateMilestoneDto,
  MilestoneDetailDto,
  MilestoneSummaryDto,
  UpdateMilestoneDto,
} from 'src/types/milestone';

export const MilestonesService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<MilestoneSummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/milestones/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  getById: async (id: string): Promise<MilestoneDetailDto> => {
    const payload: QueryPayload = {
      page: 1,
      limit: 1,
      order_by: 'created_at',
      order_asc: false,
      filters: [{ column: 'id', operator: '=', value: id }],
    };

    const response = await MilestonesService.search(payload);
    const milestone = response.items[0];

    if (!milestone) {
      throw new Error('Milestone no encontrado');
    }

    return milestone;
  },

  create: (
    data: CreateMilestoneDto,
    files: readonly MarkdownPendingFile[] = [],
  ): Promise<MilestoneDetailDto> =>
    apiFetch('/milestones', {
      method: 'POST',
      body: buildMultipartPayload(data, files),
    }),

  update: (
    id: string,
    data: UpdateMilestoneDto,
    files: readonly MarkdownPendingFile[] = [],
  ): Promise<MilestoneDetailDto> =>
    apiFetch(`/milestones/${id}`, {
      method: 'PUT',
      body: buildMultipartPayload(data, files),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/milestones/${id}`, {
      method: 'DELETE',
    }),
};
