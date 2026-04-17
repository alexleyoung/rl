// Keymap storage. Bindings shown as user-facing strings like "⌘K", "⌘⇧H", "/", "g i".
// Persisted to localStorage under `rl_keymap`. `getBindingsForAction` reads the
// override if present, else falls back to the default.

export type Binding = string;

export interface KeymapEntry {
  action: string;
  bindings: Binding[];  // multiple bindings OK (design shows e.g. j/k as two entries)
  context: string;      // human-readable context label
  group: 'global' | 'navigation' | 'in list' | 'reader';
}

export const defaultKeymap: Record<string, KeymapEntry> = {
  'command palette':    { action: 'command palette',    bindings: ['⌘K'],      context: 'anywhere',   group: 'global' },
  'add new resource':   { action: 'add new resource',   bindings: ['⌘N'],      context: 'anywhere',   group: 'global' },
  'focus filter':       { action: 'focus filter',       bindings: ['/'],       context: 'list views', group: 'global' },
  'toggle pdf pane':    { action: 'toggle pdf pane',    bindings: ['⌘.'],      context: 'reader',     group: 'global' },
  'this help':          { action: 'this help',          bindings: ['?'],       context: 'anywhere',   group: 'global' },

  'go to inbox':        { action: 'go to inbox',        bindings: ['g i'],     context: 'anywhere',   group: 'navigation' },
  'go to reading':      { action: 'go to reading',      bindings: ['g r'],     context: 'anywhere',   group: 'navigation' },
  'go to queue':        { action: 'go to queue',        bindings: ['g q'],     context: 'anywhere',   group: 'navigation' },
  'go to done':         { action: 'go to done',         bindings: ['g d'],     context: 'anywhere',   group: 'navigation' },
  'next / prev tag':    { action: 'next / prev tag',    bindings: ['[', ']'],  context: 'tag view',   group: 'navigation' },

  'move selection':     { action: 'move selection',     bindings: ['j', 'k'],  context: 'list views', group: 'in list' },
  'open resource':      { action: 'open resource',      bindings: ['↵'],       context: 'list views', group: 'in list' },
  'mark read / unread': { action: 'mark read / unread', bindings: ['e'],       context: 'list views', group: 'in list' },
  'move to queue':      { action: 'move to queue',      bindings: ['q'],       context: 'list views', group: 'in list' },
  'move to folder':     { action: 'move to folder',     bindings: ['m'],       context: 'list views', group: 'in list' },
  'add tag':            { action: 'add tag',            bindings: ['t'],       context: 'list views', group: 'in list' },
  'mark done':          { action: 'mark done',          bindings: ['⌘↵'],      context: 'list views', group: 'in list' },
  'delete':             { action: 'delete',             bindings: ['⌘⌫'],      context: 'list views', group: 'in list' },

  'swap panes':         { action: 'swap panes',         bindings: ['F'],       context: 'reader',     group: 'reader' },
  'toggle notes pane':  { action: 'toggle notes pane',  bindings: ['⌘⇧.'],     context: 'reader',     group: 'reader' },
  'highlight selection':{ action: 'highlight selection',bindings: ['⌘⇧H'],     context: 'reader',     group: 'reader' },
  'new note block':     { action: 'new note block',     bindings: ['⌘↵'],      context: 'notes pane', group: 'reader' },
  'jump to outline':    { action: 'jump to outline',    bindings: ['⌘1–9'],    context: 'reader',     group: 'reader' },
};

const STORE_KEY = 'rl_keymap';

type Overrides = Record<string, Binding[]>;

export function loadOverrides(): Overrides {
  if (typeof localStorage === 'undefined') return {};
  try {
    const raw = localStorage.getItem(STORE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

export function saveOverrides(o: Overrides): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(STORE_KEY, JSON.stringify(o));
}

export function getBindingsForAction(action: string): Binding[] {
  const entry = defaultKeymap[action];
  if (!entry) return [];
  const o = loadOverrides();
  return o[action] ?? entry.bindings;
}

export function countModified(): number {
  const o = loadOverrides();
  let n = 0;
  for (const k of Object.keys(o)) {
    const def = defaultKeymap[k];
    if (!def) continue;
    if (JSON.stringify(o[k]) !== JSON.stringify(def.bindings)) n++;
  }
  return n;
}
