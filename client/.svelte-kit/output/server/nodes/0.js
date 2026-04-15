

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false,
  "prerender": false
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.BU3U7kzI.js","_app/immutable/chunks/CMugsOeq.js","_app/immutable/chunks/DmpZ7VhR.js","_app/immutable/chunks/DbvipPTE.js","_app/immutable/chunks/vCoaxajZ.js","_app/immutable/chunks/a28l6VXo.js","_app/immutable/chunks/Cxb8ZSo8.js","_app/immutable/chunks/CjL-Xyzu.js","_app/immutable/chunks/C3kF-xe7.js"];
export const stylesheets = ["_app/immutable/assets/0.CYJrBZHp.css"];
export const fonts = [];
