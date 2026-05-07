import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types';
import type {
  AdminCreateUserDto,
  AdminUpdateUserDto,
  AdminUserSummary,
  UpdateNotificationPreferencesDto,
  UpdateUserPasswordDto,
  UpdateUserPreferencesDto,
  UpdateUserProfileDto,
  DisableTotpDto,
  TotpEnableResult,
  TotpSetup,
  TotpStatus,
  UserSettings,
  VerifyTotpCodeDto,
} from 'src/types/user';

export const UsersService = {
  search: (payload: QueryPayload): Promise<PaginatedResponse<AdminUserSummary>> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/users/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  adminCreate: (data: AdminCreateUserDto): Promise<UserSettings> =>
    apiFetch('/users', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  adminUpdate: (id: string, data: AdminUpdateUserDto): Promise<UserSettings> =>
    apiFetch(`/users/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  adminDelete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/users/${id}`, {
      method: 'DELETE',
    }),

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

  getTotpStatus: (): Promise<TotpStatus> =>
    apiFetch('/users/me/2fa/status', {
      method: 'GET',
    }),

  startTotpSetup: (): Promise<TotpSetup> =>
    apiFetch('/users/me/2fa/setup', {
      method: 'POST',
    }),

  enableTotp: (data: VerifyTotpCodeDto): Promise<TotpEnableResult> =>
    apiFetch('/users/me/2fa/enable', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  cancelTotpSetup: (): Promise<TotpStatus> =>
    apiFetch('/users/me/2fa/setup', {
      method: 'DELETE',
    }),

  disableTotp: (data: DisableTotpDto): Promise<TotpStatus> =>
    apiFetch('/users/me/2fa/disable', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
};
