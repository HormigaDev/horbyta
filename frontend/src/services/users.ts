import { apiFetch } from 'src/boot/fetch';
import type {
  UpdateNotificationPreferencesDto,
  UpdateUserPasswordDto,
  UpdateUserPreferencesDto,
  UpdateUserProfileDto,
  UserSettings,
} from 'src/types/user';

export const UsersService = {
  getMe: (): Promise<UserSettings> =>
    apiFetch('/users/me', {
      method: 'GET',
    }),

  updateProfile: (data: UpdateUserProfileDto): Promise<UserSettings> =>
    apiFetch('/users/me/profile', {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  updatePassword: (data: UpdateUserPasswordDto): Promise<{ message: string }> =>
    apiFetch('/users/me/password', {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  updatePreferences: (data: UpdateUserPreferencesDto): Promise<UserSettings> =>
    apiFetch('/users/me/preferences', {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  updateNotifications: (data: UpdateNotificationPreferencesDto): Promise<UserSettings> =>
    apiFetch('/users/me/preferences/notifications', {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),
};
