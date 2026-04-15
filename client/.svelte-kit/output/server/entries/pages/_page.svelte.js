import { c as ensure_array_like, a as attr_class, e as escape_html, b as attr, s as stringify } from "../../chunks/root.js";
import "@sveltejs/kit/internal";
import "../../chunks/exports.js";
import "../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import "../../chunks/state.svelte.js";
import "../../chunks/client.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let resources = [];
    let tags = [];
    let activeTag = null;
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <div class="tags"><!--[-->`);
    const each_array = ensure_array_like(tags);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let t = each_array[$$index];
      $$renderer2.push(`<button${attr_class("tag", void 0, { "active": activeTag === t.name })}>${escape_html(t.name)} <span class="dim">(${escape_html(t.count)})</span></button>`);
    }
    $$renderer2.push(`<!--]--></div> <table><thead><tr><th>title</th><th>kind</th><th>tags</th><th></th></tr></thead><tbody><!--[-->`);
    const each_array_1 = ensure_array_like(resources);
    for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
      let r = each_array_1[$$index_1];
      $$renderer2.push(`<tr><td><a${attr("href", `/resources/${stringify(r.id)}`)}>${escape_html(r.title)}</a></td><td><span class="kind">${escape_html(r.kind)}</span></td><td class="small dim">${escape_html(r.tags.join(", "))}</td><td><span class="row-actions"><a${attr("href", `/resources/${stringify(r.id)}/edit`)} class="btn small">edit</a> <button class="danger small">del</button></span></td></tr>`);
    }
    $$renderer2.push(`<!--]--></tbody></table>`);
  });
}
export {
  _page as default
};
