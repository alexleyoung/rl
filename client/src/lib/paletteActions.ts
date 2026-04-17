import { writable } from 'svelte/store';

export interface Action { label: string; kbd?: string; run: () => void }

export const pageActions = writable<Action[]>([]);

// Page-local handlers for named keymap actions (e.g. 'toggle pdf pane').
// fireAction in the root layout falls through to this map for actions it
// doesn't own globally, allowing pages to handle keymap entries themselves.
export const pageHandlers = writable<Record<string, () => void>>({});
