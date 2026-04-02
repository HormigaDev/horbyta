import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type { Category, CategoryDto } from 'src/types/category';

export const CategoriesService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<Category>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/categories/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  create: (data: CategoryDto): Promise<Category> =>
    apiFetch('/categories', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  getById: (id: string): Promise<Category> =>
    apiFetch(`/categories/${id}`, {
      method: 'GET',
    }),

  update: (id: string, data: CategoryDto): Promise<Category> =>
    apiFetch(`/categories/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/categories/${id}`, {
      method: 'DELETE',
    }),
};
