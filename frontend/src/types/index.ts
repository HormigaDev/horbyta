export interface RuleErrorState {
  error: boolean;
  message: string;
}

export type Rule<T> = (prop: T) => false | string;
