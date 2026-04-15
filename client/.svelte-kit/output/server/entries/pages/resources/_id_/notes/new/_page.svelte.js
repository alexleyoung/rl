import { b as attr, e as escape_html, s as stringify, d as derived } from "../../../../../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../../../../../chunks/exports.js";
import "../../../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../../../../../chunks/state.svelte.js";
import { p as page } from "../../../../../../chunks/index2.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    const rid = derived(() => Number(page.params.id));
    let title = "";
    let body_md = "";
    let saving = false;
    $$renderer2.push(`<h1>new note</h1> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <form><div><label for="title">title *</label> <input id="title" type="text"${attr("value", title)}/></div> <div><label for="body">body (markdown)</label> <textarea id="body"${attr("rows", 16)}>`);
    const $$body = escape_html(body_md);
    if ($$body) {
      $$renderer2.push(`${$$body}`);
    }
    $$renderer2.push(`</textarea></div> <div class="row-actions"><button type="submit" class="primary"${attr("disabled", saving, true)}>${escape_html("save")}</button> <a${attr("href", `/resources/${stringify(rid())}`)} class="btn">cancel</a></div></form>`);
  });
}
export {
  _page as default
};
