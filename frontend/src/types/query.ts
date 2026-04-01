export type FilterOperator = '=' | '!=' | '>' | '<' | '>=' | '<=' | '><' | '<>' | '()';

export interface Filter {
  column: string;
  operator: FilterOperator;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  value: any;
}

export interface QueryPayload {
  page: number;
  limit: number;
  order_by: string | null;
  order_asc: boolean;
  filters: Filter[];
}

export type ColumnType = 'Text' | 'Number' | 'Boolean' | 'Date';

export type FilterSchema = Record<string, ColumnType>;

export interface PaginatedResponse<T> {
  payload: QueryPayload;
  items: T[];
  total: number;
}

export interface ApiError {
  error: string;
  type: string;
}
