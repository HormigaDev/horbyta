import { apiFetchClient } from 'src/boot/fetch';
import type { ClientDashboardDto } from 'src/types/dashboard';

export const ClientDashboardService = {
  getOverview: (): Promise<ClientDashboardDto> =>
    apiFetchClient('/dashboard/client', { method: 'GET' }),
};
