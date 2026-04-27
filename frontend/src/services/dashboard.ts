import type {
  AccountsReceivableRow,
  BillableRatioDailyRow,
  DashboardDto,
  MonthlyRevenueRow,
  TaskBottleneckRow,
  TimeByActivityRow,
} from '../types/dashboard';
import type {
  AccountsReceivableFiltersDto,
  BillableRatioFiltersDto,
  MonthlyRevenueFiltersDto,
  TaskBottlenecksFiltersDto,
  TimeByActivityFiltersDto,
} from '../types/dashboardFilters';
import { apiFetch } from '../boot/fetch';

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
): Promise<TResult> => apiFetch(`${url}${buildFilterQuery(filters)}`, { method: 'GET' });

export const DashboardService = {
  getOverview: (): Promise<DashboardDto> => apiFetch('/dashboard', { method: 'GET' }),

  getMonthlyRevenue: (filters: MonthlyRevenueFiltersDto): Promise<MonthlyRevenueRow[]> =>
    getFilters('/dashboard/charts/monthly-revenue', filters),

  getAccountsReceivable: (filters: AccountsReceivableFiltersDto): Promise<AccountsReceivableRow[]> =>
    getFilters('/dashboard/charts/accounts-receivable', filters),

  getTimeByActivity: (filters: TimeByActivityFiltersDto): Promise<TimeByActivityRow[]> =>
    getFilters('/dashboard/charts/time-by-activity', filters),

  getBillableRatio: (filters: BillableRatioFiltersDto): Promise<BillableRatioDailyRow[]> =>
    getFilters('/dashboard/charts/billable-ratio', filters),

  getTaskBottlenecks: (filters: TaskBottlenecksFiltersDto): Promise<TaskBottleneckRow[]> =>
    getFilters('/dashboard/charts/task-bottlenecks', filters),
};
