import type { Filter, QueryPayload } from '../types/query';

type TableRequestProps = {
  pagination: {
    page: number;
    rowsPerPage: number;
    sortBy: string | null;
    descending: boolean;
  };
};

interface UseCrudListHandlersOptions {
  query: QueryPayload;
  fetch: () => Promise<void>;
}

export function useCrudListHandlers(options: UseCrudListHandlersOptions) {
  const onRequest = (props: Record<string, unknown>) => {
    const pagination = (props as TableRequestProps).pagination;

    if (!pagination) {
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

    if (shouldFetch) {
      void options.fetch();
    }
  };

  const onFilters = (filters: Filter[]) => {
    options.query.filters = filters;
    options.query.page = 1;
    void options.fetch();
  };

  return {
    onRequest,
    onFilters,
  };
}
