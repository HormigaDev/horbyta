import type { FilterSchema } from './query';

export const CATEGORY_FILTER_SCHEMA: FilterSchema = {
  id: 'Text',
  name: 'Text',
};

export const CLIENT_FILTER_SCHEMA: FilterSchema = {
  id: 'Text',
  name: 'Text',
  status: 'Text',
  created_at: 'Date',
  updated_at: 'Date',
};
