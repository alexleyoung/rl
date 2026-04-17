export const STATUSES = ['inbox', 'reading', 'queue', 'done'] as const;
export type Status = typeof STATUSES[number];
