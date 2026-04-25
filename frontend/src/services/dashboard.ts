import type { DashboardDto } from '../types/dashboard';
import { apiFetch } from '../boot/fetch';

export const DashboardService = {
  getOverview: (): Promise<DashboardDto> => apiFetch('/dashboard', { method: 'GET' }),
};
