import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type {
  CategoryDetailDto,
  CategorySummaryDto,
  CreateCategoryDto,
  UpdateCategoryDto,
} from 'src/types/category';

export const CategoriesService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<CategorySummaryDto>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/categories/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: CreateCategoryDto): Promise<CategoryDetailDto> =>
    apiFetch('/categories', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<CategoryDetailDto> =>
    apiFetch(`/categories/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: UpdateCategoryDto): Promise<CategoryDetailDto> =>
    apiFetch(`/categories/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/categories/${id}`, {
      method: 'DELETE',
    }),
};
