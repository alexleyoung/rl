
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	type MatcherParam<M> = M extends (param : string) => param is (infer U extends string) ? U : string;

	export interface AppTypes {
		RouteId(): "/" | "/resources" | "/resources/new" | "/resources/[id]" | "/resources/[id]/edit" | "/resources/[id]/notes" | "/resources/[id]/notes/new" | "/resources/[id]/notes/[nid]" | "/resources/[id]/notes/[nid]/edit" | "/search";
		RouteParams(): {
			"/resources/[id]": { id: string };
			"/resources/[id]/edit": { id: string };
			"/resources/[id]/notes": { id: string };
			"/resources/[id]/notes/new": { id: string };
			"/resources/[id]/notes/[nid]": { id: string; nid: string };
			"/resources/[id]/notes/[nid]/edit": { id: string; nid: string }
		};
		LayoutParams(): {
			"/": { id?: string; nid?: string };
			"/resources": { id?: string; nid?: string };
			"/resources/new": Record<string, never>;
			"/resources/[id]": { id: string; nid?: string };
			"/resources/[id]/edit": { id: string };
			"/resources/[id]/notes": { id: string; nid?: string };
			"/resources/[id]/notes/new": { id: string };
			"/resources/[id]/notes/[nid]": { id: string; nid: string };
			"/resources/[id]/notes/[nid]/edit": { id: string; nid: string };
			"/search": Record<string, never>
		};
		Pathname(): "/" | "/resources/new" | `/resources/${string}` & {} | `/resources/${string}/edit` & {} | `/resources/${string}/notes/new` & {} | `/resources/${string}/notes/${string}` & {} | `/resources/${string}/notes/${string}/edit` & {} | "/search";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): string & {};
	}
}