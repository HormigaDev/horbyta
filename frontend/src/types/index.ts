import type { ComposerTranslation } from 'vue-i18n';

export interface RuleErrorState {
  error: boolean;
  message: string;
}

export type Rule<T> = (prop: T, t: ComposerTranslation) => false | string;

export * from './query';
export * from './project';
export * from './task';
export * from './attachment';
export * from './chat';
export * from './user';
export * from './clientAuth';
export * from './changeRequest';
