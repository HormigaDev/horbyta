import { apiFetchClient } from 'src/boot/fetch';
import type {
  ClientChangeRequestStatusRow,
  ClientDashboardDto,
  ClientMilestoneHealthRow,
  ClientProjectPortfolioRow,
  ClientProjectStatusRow,
} from 'src/types/dashboard';
import type {
  ClientChangeRequestStatusFiltersDto,
  ClientMilestoneHealthFiltersDto,
  ClientProjectPortfolioFiltersDto,
  ClientProjectStatusFiltersDto,
} from 'src/types/dashboardFilters';

type FilterValue = string | number | boolean | null | undefined;

const buildFilterQuery = <TFilters extends object>(filters: TFilters) => {
  const params = new URLSearchParams();

  Object.entries(filters as Record<string, FilterValue>).forEach(([key, value]) => {
    if (value === null || value === undefined || value === '') {
      return;
    }

    params.set(key, String(value));
  });

  const query = params.toString();
  return query.length > 0 ? `?${query}` : '';
};

const getFilters = <TResult>(
  url: string,
  filters: object,
): Promise<TResult> => apiFetchClient(`${url}${buildFilterQuery(filters)}`, { method: 'GET' });

export const ClientDashboardService = {
  getOverview: (): Promise<ClientDashboardDto> =>
    apiFetchClient('/dashboard/client', { method: 'GET' }),

  getProjectStatus: (filters: ClientProjectStatusFiltersDto): Promise<ClientProjectStatusRow[]> =>
    getFilters('/dashboard/client/charts/project-status', filters),

  getMilestoneHealth: (
    filters: ClientMilestoneHealthFiltersDto,
  ): Promise<ClientMilestoneHealthRow[]> =>
    getFilters('/dashboard/client/charts/milestone-health', filters),

  getChangeRequestStatus: (
    filters: ClientChangeRequestStatusFiltersDto,
  ): Promise<ClientChangeRequestStatusRow[]> =>
    getFilters('/dashboard/client/charts/change-request-status', filters),

  getProjectPortfolio: (
    filters: ClientProjectPortfolioFiltersDto,
  ): Promise<ClientProjectPortfolioRow[]> =>
    getFilters('/dashboard/client/charts/project-portfolio', filters),
};
