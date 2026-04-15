import "clsx";
import "@sveltejs/kit/internal";
import "../../../../chunks/exports.js";
import "../../../../chunks/utils.js";
import "@sveltejs/kit/internal/server";
import { b as attr, e as escape_html } from "../../../../chunks/root.js";
import "../../../../chunks/state.svelte.js";
function ResourceForm($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { submitLabel = "save" } = $$props;
    let kind = "book";
    let title = "";
    let author = "";
    let url = "";
    let file_path = "";
    let tagsStr = "";
    let saving = false;
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <form><div><label for="kind">kind</label> `);
    $$renderer2.select({ id: "kind", value: kind }, ($$renderer3) => {
      $$renderer3.option({}, ($$renderer4) => {
        $$renderer4.push(`book`);
      });
      $$renderer3.option({}, ($$renderer4) => {
        $$renderer4.push(`paper`);
      });
      $$renderer3.option({}, ($$renderer4) => {
        $$renderer4.push(`article`);
      });
      $$renderer3.option({}, ($$renderer4) => {
        $$renderer4.push(`blog`);
      });
      $$renderer3.option({}, ($$renderer4) => {
        $$renderer4.push(`repo`);
      });
    });
    $$renderer2.push(`</div> <div><label for="title">title *</label> <input id="title" type="text"${attr("value", title)}/></div> <div><label for="author">author</label> <input id="author" type="text"${attr("value", author)}/></div> <div><label for="url">url</label> <input id="url" type="url"${attr("value", url)}/></div> <div><label for="file_path">file path</label> <input id="file_path" type="text"${attr("value", file_path)} placeholder="/path/to/file.pdf"/></div> <div><label for="tags">tags (comma-separated)</label> <input id="tags" type="text"${attr("value", tagsStr)} placeholder="algorithms, systems"/></div> <div class="row-actions"><button type="submit" class="primary"${attr("disabled", saving, true)}>${escape_html(submitLabel)}</button> <button type="button">cancel</button></div></form>`);
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    $$renderer2.push(`<h1>add resource</h1> `);
    ResourceForm($$renderer2, { submitLabel: "add" });
    $$renderer2.push(`<!---->`);
  });
}
export {
  _page as default
};
