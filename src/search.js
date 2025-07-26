import { marked } from 'marked';
import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";

const first_element = document.getElementById('leftnav');

console.log("Hello?");
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
document.body.insertBefore(context, first_element);

let results = [];
let fuse;

const toggle_picker = async () => {
  if (picker.style.display === "block") {
    picker.style.display = "none";
  } else {
    picker.style.display = "block";
    input.focus();
  }
}

const toggle_overlay = async () => {
  const context = document.getElementById("snippet-context");
  if (context.style.display === "block") {
    context.style.display = "none";
  } else {
    context.style.display = "block";
  }
}

document.addEventListener('DOMContentLoaded', async () => {
  const corpus = await invoke('load_snippets')
    .catch((error) => console.log("error caught:" + error));

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
    console.log("Loading picker for marked docs");
    toggle_picker();
    input.onkeydown = (e) => mark_document_bind(e);
  }
  if (e.ctrlKey && e.key === "f") {
    console.log("Loading picker for local docs");
    toggle_picker();
    input.onkeydown = (e) => load_document_bind(e);
  }
}

document.getElementById("marked_document").onclick = function() {
  toggle_picker();
  input.onkeydown = (e) => mark_document_bind(e);
};

document.getElementById("current_document").onclick = function() {
  toggle_picker();
  input.onkeydown = (e) => load_document_bind(e);
};

const load_document_bind = (e) => {
  if (e.key === "Enter") {
    let document_name = "None";
    if (results.length > 0 && Array.isArray(results)) {
      console.log("Results found");
      document_name = results[0].item;
    } else {
      console.log("No results found");
    }
    toggle_picker();
    if (document.body.id === "view") {
      update_view(document_name);
    }

    document.getElementById("current_document").innerText = document_name;
    localStorage['current_document'] = document_name;
    input.value = "";
  }
}

const mark_document_bind = (e) => {
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
        //  const marked_document = await invoke('fetch_marked_document')
        //    .catch((error) => console.log("Error caught:" + error));
        //  console.log(marked_document);
      }
    });
    document.getElementById("marked_document").innerText = document_name;
    input.value = "";
  }
}

const snippets = [];
const update_view = async (search_document) => {
  let snippet_container = document.getElementById('snippet');
  snippet_container.innerHTML = "";

  const viewed_document = await invoke('load_document', { documentName: search_document })
    .catch((error) => console.log("Error caught:" + error));
  let document_name = "None";

  if (viewed_document != null) {
    document_name = viewed_document.document_name;
    viewed_document.snippets.forEach(snippet =>
      snippets.push({
        document_name: viewed_document.document_name,
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
  console.log("Firing toggle function");
  toggle_overlay();

  edit_card.onblur = () => {
    saveSnippet(edit_card, view_card.id);
    toggle_overlay();
  }

  document.getElementById("update_snippet").onclick = () => saveSnippet(edit_card, id);
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

const deleteSnippet = (id) => {
  const snippet = snippets.pop(i => i.snippet_id === parseInt(id));
  invoke('remove_snippet', { snippetId: parseInt(id) });
}
