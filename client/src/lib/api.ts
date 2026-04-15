import type { ResourceDto } from './types/ResourceDto';
import type { ResourceInputDto } from './types/ResourceInputDto';
import type { ResourceDetailDto } from './types/ResourceDetailDto';
import type { NoteDto } from './types/NoteDto';
import type { NoteInputDto } from './types/NoteInputDto';
import type { NoteLocationDto } from './types/NoteLocationDto';
import type { QuickSetDto } from './types/QuickSetDto';
import type { TagDto } from './types/TagDto';
import type { SearchResponseDto } from './types/SearchResponseDto';
import type { ApiErrorDto } from './types/ApiErrorDto';
import type { UploadResponseDto } from './types/UploadResponseDto';

export type {
  ResourceDto, ResourceInputDto, ResourceDetailDto,
  NoteDto, NoteInputDto, NoteLocationDto,
  QuickSetDto, TagDto, SearchResponseDto, ApiErrorDto, UploadResponseDto,
};

const BASE = '/api/v1';

export class ApiError extends Error {
  constructor(public status: number, public code: string, message: string) {
    super(message);
  }
}

async function req<T>(path: string, init?: RequestInit): Promise<T> {
  const r = await fetch(BASE + path, {
    headers: { 'content-type': 'application/json', ...init?.headers },
    ...init,
  });
  if (!r.ok) {
    let code = 'internal';
    let message = `HTTP ${r.status}`;
    try {
      const err: ApiErrorDto = await r.json();
      code = err.code;
      message = err.message;
    } catch {}
    throw new ApiError(r.status, code, message);
  }
  if (r.status === 204) return undefined as T;
  return r.json();
}

export const api = {
  // Resources
  listResources: (tag?: string) =>
    req<ResourceDto[]>(`/resources${tag ? `?tag=${encodeURIComponent(tag)}` : ''}`),

  getResource: (id: number) =>
    req<ResourceDetailDto>(`/resources/${id}`),

  createResource: (body: ResourceInputDto) =>
    req<ResourceDto>('/resources', { method: 'POST', body: JSON.stringify(body) }),

  updateResource: (id: number, body: ResourceInputDto) =>
    req<ResourceDto>(`/resources/${id}`, { method: 'PATCH', body: JSON.stringify(body) }),

  deleteResource: (id: number) =>
    req<void>(`/resources/${id}`, { method: 'DELETE' }),

  quickSet: (id: number, body: QuickSetDto) =>
    req<ResourceDto>(`/resources/${id}/quick-set`, { method: 'POST', body: JSON.stringify(body) }),

  fileUrl: (id: number) => `${BASE}/resources/${id}/file`,

  // Notes
  listNotes: (rid: number) =>
    req<NoteDto[]>(`/resources/${rid}/notes`),

  getNote: (rid: number, nid: number) =>
    req<NoteDto>(`/resources/${rid}/notes/${nid}`),

  createNote: (rid: number, body: NoteInputDto) =>
    req<NoteDto>(`/resources/${rid}/notes`, { method: 'POST', body: JSON.stringify(body) }),

  updateNote: (rid: number, nid: number, body: NoteInputDto) =>
    req<NoteDto>(`/resources/${rid}/notes/${nid}`, { method: 'PATCH', body: JSON.stringify(body) }),

  deleteNote: (rid: number, nid: number) =>
    req<void>(`/resources/${rid}/notes/${nid}`, { method: 'DELETE' }),

  locateNote: (nid: number) =>
    req<NoteLocationDto>(`/notes/${nid}`),

  // Tags
  listTags: () => req<TagDto[]>('/tags'),

  // Search
  search: (q: string, limit?: number) =>
    req<SearchResponseDto>(`/search?q=${encodeURIComponent(q)}${limit ? `&limit=${limit}` : ''}`),

  // Upload a file, returns the server-side absolute path
  uploadFile: async (file: File): Promise<UploadResponseDto> => {
    const form = new FormData();
    form.append('file', file);
    const r = await fetch(`${BASE}/upload`, { method: 'POST', body: form });
    if (!r.ok) {
      let code = 'internal', message = `HTTP ${r.status}`;
      try { const err: ApiErrorDto = await r.json(); code = err.code; message = err.message; } catch {}
      throw new ApiError(r.status, code, message);
    }
    return r.json();
  },
};
