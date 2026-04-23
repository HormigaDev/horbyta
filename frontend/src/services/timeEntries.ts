import { apiFetch } from 'src/boot/fetch';
import type { QueryPayload } from 'src/types';
import type {
  ActiveTimeEntryTimerResponse,
  CreateTimeEntryDto,
  StartTimeEntryTimerDto,
  StopTimeEntryTimerDto,
  TimeEntryDetailDto,
  TimeEntrySearchResponse,
  UpdateTimeEntryDto,
} from 'src/types/timeEntry';

export const TimeEntriesService = {
  search: (payload: QueryPayload): Promise<TimeEntrySearchResponse> => {
    const params = new URLSearchParams({ q: JSON.stringify(payload) });
    return apiFetch(`/time-entries/search?${params.toString()}`, {
      method: 'GET',
    });
  },

  getById: (id: string): Promise<TimeEntryDetailDto> =>
    apiFetch(`/time-entries/${id}`, {
      method: 'GET',
    }),

  createManual: (data: CreateTimeEntryDto): Promise<TimeEntryDetailDto> =>
    apiFetch('/time-entries', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  update: (id: string, data: UpdateTimeEntryDto): Promise<TimeEntryDetailDto> =>
    apiFetch(`/time-entries/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  startTimer: (data: StartTimeEntryTimerDto): Promise<TimeEntryDetailDto> =>
    apiFetch('/time-entries/timer/start', {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  stopTimer: (id: string, data: StopTimeEntryTimerDto = {}): Promise<TimeEntryDetailDto> =>
    apiFetch(`/time-entries/${id}/timer/stop`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    }),

  getActiveTimer: (): Promise<ActiveTimeEntryTimerResponse> =>
    apiFetch('/time-entries/timer/active', {
      method: 'GET',
    }),

  delete: (id: string): Promise<{ message: string }> =>
    apiFetch(`/time-entries/${id}`, {
      method: 'DELETE',
    }),
};
