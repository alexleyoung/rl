import { a as attr_class, b as attr } from "../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../chunks/exports.js";
import "../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../chunks/state.svelte.js";
import { p as page } from "../../chunks/index2.js";
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { children } = $$props;
    let searchQuery = "";
    $$renderer2.push(`<div id="wrap"><nav><a href="/" class="brand">rl</a> <a href="/"${attr_class("", void 0, {
      "active": (
        // Sync search input from URL on search page
        page.url.pathname === "/"
      )
    })}>library</a> <a href="/resources/new">+ add</a></nav> <div id="search-bar"><form style="flex-direction:row; gap:0.5rem;"><input type="text" placeholder="search…"${attr("value", searchQuery)}/> <button type="submit">search</button></form></div> `);
    children($$renderer2);
    $$renderer2.push(`<!----></div>`);
  });
}
export {
  _layout as default
};
