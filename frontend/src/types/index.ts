export interface RuleErrorState {
  error: boolean;
  message: string;
}

export type Rule<T> = (prop: T) => false | string;

export interface User {
  id: string;
  name: string;
  email: string;
  created_at: string;
  updated_at: string;
}

export * from './query';
export * from './project';
export * from './task';
export * from './chat';
