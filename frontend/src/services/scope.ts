import { apiFetch, apiFetchClient } from 'src/boot/fetch';
import type { ConversationNotice } from 'src/stores/conversation';
import type { ConversationResource } from 'src/types/scope';
import type { AttachmentUploadResponseDto } from 'src/types/attachment';
import type {
  ScopeProject,
  ScopeDocument,
  ScopeChange,
  ScopeTask,
  ScopeNotice,
  ScopeConversationWindow,
} from 'src/types/scope';
const fetcher = (client: boolean) => (client ? apiFetchClient : apiFetch);
export const ScopeService = {
  conversationInbox: (client = false): Promise<ConversationNotice[]> =>
    fetcher(client)(`/scope/${client ? 'client/' : ''}conversation/inbox`),
  projectConversation: (
    id: string,
    client = false,
    cursor: { before?: string; after?: string; around?: string; notification?: string } = {},
  ): Promise<ScopeConversationWindow> =>
    fetcher(client)(
      `/scope/${client ? 'client/' : ''}projects/${id}/conversation?${new URLSearchParams(cursor)}`,
    ),
  sendProjectMessage: (id: string, body: object, client = false): Promise<{ id: string }> =>
    fetcher(client)(`/scope/${client ? 'client/' : ''}projects/${id}/conversation`, {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  readConversation: (id: string, message_id: string, client = false) =>
    fetcher(client)(`/scope/${client ? 'client/' : ''}projects/${id}/conversation/read`, {
      method: 'POST',
      body: JSON.stringify({ message_id }),
    }),
  conversationResources: (
    id: string,
    search: string,
    client = false,
    kind: 'resources' | 'files' = 'resources',
  ): Promise<ConversationResource[]> =>
    fetcher(client)(
      `/scope/${client ? 'client/' : ''}projects/${id}/conversation/resources?${new URLSearchParams({ search, kind })}`,
    ),
  proposeFromMessage: (
    id: string,
    message: string,
    title: string,
    description: string,
    client = false,
  ): Promise<{ id: string }> =>
    fetcher(client)(
      `/scope/${client ? 'client/' : ''}projects/${id}/conversation/${message}/change`,
      { method: 'POST', body: JSON.stringify({ title, description }) },
    ),
  uploadProjectConversation: (
    id: string,
    files: File[],
    client = false,
  ): Promise<AttachmentUploadResponseDto> => {
    const body = new FormData();
    for (const file of files) body.append('files', file);
    return fetcher(client)(`/scope/${client ? 'client/' : ''}projects/${id}/conversation/files`, {
      method: 'POST',
      body,
    });
  },
  project: (id: string, client = false): Promise<ScopeProject> =>
    fetcher(client)(`/scope/${client ? 'client/' : ''}projects/${id}`),
  publish: (
    id: string,
    expected_revision: number,
    document: ScopeDocument,
    publish: boolean,
  ): Promise<ScopeProject> =>
    apiFetch(`/scope/projects/${id}`, {
      method: 'POST',
      body: JSON.stringify({
        expected_revision,
        document: {
          ...document,
          documents: (document.documents ?? []).map((d) => ({
            id: d.id,
            title: d.title,
            role: d.role,
            section: d.section,
            content: d.content,
            source_name: d.source_name,
            source_content: d.source_content,
            requirement_id: d.requirement_id,
          })),
        },
        publish,
      }),
    }),
  accept: (id: string, revision: number): Promise<ScopeProject> =>
    apiFetchClient(`/scope/client/projects/${id}/accept`, {
      method: 'POST',
      body: JSON.stringify({ revision }),
    }),
  change: (id: string, client = false): Promise<ScopeChange> =>
    fetcher(client)(`/scope/${client ? 'client/' : ''}changes/${id}`),
  comment: (
    id: string,
    version: number,
    text: string,
    client = false,
    attachment_ids: string[] = [],
  ): Promise<ScopeChange> =>
    fetcher(client)(`/scope/${client ? 'client/' : ''}changes/${id}/comments`, {
      method: 'POST',
      body: JSON.stringify({ version, text, attachment_ids }),
    }),
  task: (id: string): Promise<ScopeTask> => apiFetch(`/scope/tasks/${id}`),
  classify: (id: string, change_request_id: string | null, reason: string): Promise<ScopeTask> =>
    apiFetch(`/scope/tasks/${id}`, {
      method: 'POST',
      body: JSON.stringify({ change_request_id, reason }),
    }),
  feedback: (
    id: string,
    revision: number,
    kind: string,
    text: string,
    client = false,
    document_id: string | null = null,
    attachment_ids: string[] = [],
  ): Promise<ScopeProject> =>
    fetcher(client)(`/scope/${client ? 'client/' : ''}projects/${id}/feedback`, {
      method: 'POST',
      body: JSON.stringify({ revision, kind, text, document_id, attachment_ids }),
    }),
  changeConversation: (
    id: string,
    revision: number,
    client = false,
    cursor: { before?: string; after?: string; around?: string } = {},
  ): Promise<ScopeConversationWindow> =>
    fetcher(client)(
      `/scope/${client ? 'client/' : ''}changes/${id}/conversation/${revision}?${new URLSearchParams(cursor)}`,
    ),
  uploadChangeConversation: (
    id: string,
    revision: number,
    files: File[],
    client = false,
  ): Promise<AttachmentUploadResponseDto> => {
    const body = new FormData();
    for (const file of files) body.append('files', file);
    return fetcher(client)(
      `/scope/${client ? 'client/' : ''}changes/${id}/conversation/${revision}/files`,
      { method: 'POST', body },
    );
  },
  conversation: (
    id: string,
    revision: number,
    client = false,
    cursor: { before?: string; after?: string; around?: string; notification?: string } = {},
  ): Promise<ScopeConversationWindow> =>
    fetcher(client)(
      `/scope/${client ? 'client/' : ''}projects/${id}/conversation/${revision}?${new URLSearchParams(cursor)}`,
    ),
  uploadConversation: (id: string, revision: number, files: File[], client = false) => {
    const body = new FormData();
    for (const file of files) body.append('files', file);
    return fetcher(client)(
      `/scope/${client ? 'client/' : ''}projects/${id}/conversation/${revision}/files`,
      { method: 'POST', body },
    ) as Promise<AttachmentUploadResponseDto>;
  },
  preparation: (id: string, reason: string): Promise<ScopeTask> =>
    apiFetch(`/scope/tasks/${id}/preparation`, {
      method: 'POST',
      body: JSON.stringify({ reason }),
    }),
  requirement: (id: string, document_id: string, revision: number): Promise<ScopeTask> =>
    apiFetch(`/scope/tasks/${id}/requirements`, {
      method: 'POST',
      body: JSON.stringify({ document_id, revision }),
    }),
  pending: (): Promise<ScopeNotice[]> => apiFetchClient('/scope/client/pending'),
};
