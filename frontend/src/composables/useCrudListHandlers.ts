import type { Filter, QueryPayload } from '../types/query';

type TableRequestProps = {
  pagination: {
    page: number;
    rowsPerPage: number;
    sortBy: string | null;
    descending: boolean;
  };
  filters?: Filter[];
};

interface UseCrudListHandlersOptions {
  query: QueryPayload;
  fetch: () => Promise<void>;
}

export function useCrudListHandlers(options: UseCrudListHandlersOptions) {
  let hasRequestedOnce = false;

  const areFiltersEqual = (left: Filter[], right: Filter[]) => {
    return JSON.stringify(left) === JSON.stringify(right);
  };

  const onRequest = (props: Record<string, unknown>) => {
    const requestProps = props as TableRequestProps;
    const pagination = requestProps.pagination;

    if (!pagination) {
      hasRequestedOnce = true;
      void options.fetch();
      return;
    }

    let shouldFetch = false;

    if (pagination.rowsPerPage && pagination.rowsPerPage !== options.query.limit) {
      options.query.limit = pagination.rowsPerPage;
      options.query.page = 1;
      shouldFetch = true;
    }

    if (pagination.page && pagination.page !== options.query.page) {
      options.query.page = pagination.page;
      shouldFetch = true;
    }

    if (pagination.sortBy) {
      const orderAsc = !pagination.descending;
      if (options.query.order_by !== pagination.sortBy || options.query.order_asc !== orderAsc) {
        options.query.order_by = pagination.sortBy;
        options.query.order_asc = orderAsc;
        shouldFetch = true;
      }
    } else {
      shouldFetch = true;
    }

    const incomingFilters = Array.isArray(requestProps.filters) ? requestProps.filters : [];
    if (!areFiltersEqual(options.query.filters, incomingFilters)) {
      options.query.filters = incomingFilters;
      shouldFetch = true;
    }

    if (!hasRequestedOnce) {
      shouldFetch = true;
    }

    if (shouldFetch) {
      hasRequestedOnce = true;
      void options.fetch();
    }
  };

  return {
    onRequest,
  };
}
