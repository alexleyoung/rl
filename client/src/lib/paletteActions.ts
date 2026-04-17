import { writable } from 'svelte/store';

export interface Action { label: string; kbd?: string; run: () => void }

export const pageActions = writable<Action[]>([]);
