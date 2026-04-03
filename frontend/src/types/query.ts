export type FilterOperator = '=' | '!=' | '>' | '<' | '>=' | '<=' | '><' | '<>' | '()';

export interface Filter {
  column: string;
  operator: FilterOperator;
  value: unknown;
}

export interface QueryPayload {
  page: number;
  limit: number;
  order_by: string | null;
  order_asc: boolean;
  filters: Filter[];
}

export type ColumnType = 'Text' | 'Number' | 'Boolean' | 'Date';

export interface FilterFieldConfig {
  type: ColumnType;
  label?: string;
  useSelectByTerm?: boolean;
}

export type FilterSchema = Record<string, ColumnType | FilterFieldConfig>;

export interface PaginatedResponse<T> {
  payload: QueryPayload;
  items: T[];
  total: number;
}

export interface ApiError {
  error: string;
  type: string;
}
