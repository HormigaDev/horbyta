import { apiFetch } from 'src/boot/fetch';
import type { PaginatedResponse, QueryPayload } from 'src/types/query';
import type {
  AttachmentStorageUsageDto,
  AttachmentSummaryDto,
  AttachmentUploadResponseDto,
} from 'src/types/attachment';

const buildSearchParams = (payload: QueryPayload): string => {
  const params = new URLSearchParams({ q: JSON.stringify(payload) });
  return params.toString();
};

const buildFilesFormData = (files: readonly File[]): FormData => {
  const formData = new FormData();

  files.forEach((file) => {
    formData.append('files', file);
  });

  return formData;
};

export const AttachmentsService = {
  searchByProject: (
    projectId: string,
    payload: QueryPayload,
  ): Promise<PaginatedResponse<AttachmentSummaryDto>> => {
    const params = buildSearchParams(payload);

    return apiFetch(`/attachments/project/${projectId}/search?${params}`, {
      method: 'GET',
    });
  },

  searchByTask: (
    taskId: string,
    payload: QueryPayload,
  ): Promise<PaginatedResponse<AttachmentSummaryDto>> => {
    const params = buildSearchParams(payload);

    return apiFetch(`/attachments/task/${taskId}/search?${params}`, {
      method: 'GET',
    });
  },

  uploadToProject: (
    projectId: string,
    files: readonly File[],
  ): Promise<AttachmentUploadResponseDto> =>
    apiFetch(`/attachments/project/${projectId}/files`, {
      method: 'POST',
      body: buildFilesFormData(files),
    }),

  uploadToTask: (taskId: string, files: readonly File[]): Promise<AttachmentUploadResponseDto> =>
    apiFetch(`/attachments/task/${taskId}/files`, {
      method: 'POST',
      body: buildFilesFormData(files),
    }),

  getStorageUsage: (): Promise<AttachmentStorageUsageDto> =>
    apiFetch('/attachments/usage', {
      method: 'GET',
    }),

  delete: (attachmentId: string): Promise<{ message: string }> =>
    apiFetch(`/attachments/${attachmentId}`, {
      method: 'DELETE',
    }),

  buildRawUrl: (attachmentId: string): string => `/api/v1/attachments/${attachmentId}/raw`,
};
