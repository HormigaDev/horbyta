export interface RuleErrorState {
  error: boolean;
  message: string;
}

export type Rule<T> = (prop: T) => false | string;

export * from './query';
export * from './project';
export * from './task';
export * from './chat';
export * from './user';
export * from './clientAuth';
