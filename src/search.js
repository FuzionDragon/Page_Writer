import { marked } from 'marked';
import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";

const first_element = document.getElementById('leftnav');
const picker_button = document.getElementById('toggle-picker');

const picker = document.createElement('div');
picker.id = "document-picker";
picker.className = "overlay-document-picker";

const input = document.createElement('input');
input.type = "text";
input.placeholder = "Searching documents... ";
input.id = "document_input";

const document_list = document.createElement('ul');
document_list.id = "document_list";

picker.appendChild(input);
picker.appendChild(document_list);

document.body.insertBefore(picker, first_element);

let results = [];
let fuse;

const toggle_picker = async () => {
  const document_picker = document.getElementById("document-picker");
  if (document_picker.style.display === "block") {
    document_picker.style.display = "none";
  } else {
    document_picker.style.display = "block";
    input.focus();
  }
}

document.addEventListener('DOMContentLoaded', async () => {
  const corpus = await invoke('load_snippets')
    .catch((error) => console.log("error caught:" + error));

  console.log("Object: " + corpus);
  fuse = new Fuse(Object.keys(corpus), {
    keys: ['title'],
    threshold: 0.4
  });
})

document.oninput = function(e) {
  document_list.innerHTML = '';
  const query = e.target.value;
  results = fuse.search(query, {
    limit: 5,
  });

  results.forEach(result => {
    console.log(result.item);
    const doc = document.createElement('li');
    doc.innerText = result.item;
    document_list.appendChild(doc);
  });
};

document.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "e") {
    toggle_picker();
  }
}

document.getElementById("marked_document").onclick = function() {
  toggle_picker();
};

input.onkeydown = function(e) {
  if (e.key === "Enter") {
    let document_name = "None";
    if (results.length > 0 && Array.isArray(results)) {
      console.log("Results found");
      document_name = results[0].item;
    } else {
      console.log("No results found");
    }

    toggle_picker();
    invoke("mark_document", { documentName: document_name }).then(() => {
      if (document.body.id === "view") {
        console.log("Updating view");
        update_view();
      }
    });
    document.getElementById("marked_document").innerText = document_name;
    input.value = "";
  }
}

const snippets = [];
const update_view = async () => {
  let snippet_container = document.getElementById('snippet');
  snippet_container.innerHTML = "";
  const marked_document = await invoke('fetch_marked_document')
    .catch((error) => console.log("Error caught:" + error));
  console.log(marked_document);
  let document_name = "None";


  if (marked_document != null) {
    document_name = marked_document.document_name;
    marked_document.snippets.forEach(snippet =>
      snippets.push({
        document_name: marked_document.document_name,
        snippet_id: snippet.snippet_id,
        raw: snippet.snippet,
        markdown: marked.parse(snippet.snippet),
      })
    );
  }

  const document_title = document.createElement('h1');
  document_title.innerText = document_name;

  snippet_container.appendChild(document_title);

  for (const snippet of snippets.entries()) {
    const view_card = document.createElement('div');
    view_card.innerHTML = snippet[1].markdown;
    view_card.id = snippet[1].snippet_id;
    view_card.onclick = () => editSnippet(view_card);

    snippet_container.appendChild(view_card);
  }
}

const editSnippet = (view_card) => {
  const edit_card = document.createElement('textarea');
  const id = view_card.id;
  const snippet = snippets.find(i => i.snippet_id === parseInt(id));
  edit_card.id = id;
  edit_card.value = snippet.raw;

  view_card.replaceWith(edit_card);
  edit_card.focus();

  edit_card.onblur = () => saveSnippet(edit_card, view_card.id);
  edit_card.onkeydown = (e) => {
    if (e.ctrlKey && e.key === "Enter") {
      saveSnippet(edit_card, id);
    }
  }
}

const saveSnippet = (edit_card, id) => {
  const content = marked.parse(edit_card.value);
  const snippet = snippets.find(i => i.snippet_id === parseInt(id));
  snippet.raw = edit_card.value;
  snippet.markdown = content;

  const view_card = document.createElement('div');
  view_card.innerHTML = content;
  view_card.id = id;
  invoke('update', { snippetId: parseInt(id), snippet: edit_card.value, documentName: snippet.document_name });
  view_card.onclick = () => editSnippet(view_card);
  edit_card.replaceWith(view_card);
}
