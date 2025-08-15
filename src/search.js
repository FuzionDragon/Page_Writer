import { marked, parse } from 'marked';
import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";
import Toastify from 'toastify-js'

import { keybind_handler } from './config';

const picker_container = document.createElement('div');
picker_container.id = "picker_container";

const picker = document.createElement('div');
picker.id = "document-picker";
picker.className = "overlay-document-picker";

const current_action = document.createElement('h3');
current_action.innerText = "None";
current_action.id = "picker_action";

const input = document.createElement('input');
input.type = "text";
input.placeholder = "Searching documents... ";
input.id = "document_input";

const document_list = document.createElement('ul');
document_list.id = "document_list";

picker.appendChild(current_action);
picker.appendChild(input);
picker.appendChild(document_list);

document.getElementById("navbar").appendChild(picker);

let results = [];
let fuse;

export const toggle_picker = async () => {
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
  invoke('fetch_marked_document')
    .then((marked_document) => {
      console.log(marked_document);
      if (marked_document === null) {
        document.getElementById('marked_document').innerText = "None";
      } else {
        document.getElementById('marked_document').innerText = marked_document;
      }
    })
    .catch((error) => console.log("Error after fetch_marked_document caught: " + error));

  if (document.body.id === "view") {
    document.getElementById("delete_document").onclick = () => deleteDocument();
    document.getElementById("delete_current_document").onclick = () => deleteCurrentDocument();
  }

  const corpus = await invoke('load_snippets')
    .catch((error) => console.log("error caught:" + error));

  localStorage['corpus'] = corpus;

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
    const doc = document.createElement('li');
    doc.innerText = result.item;
    document_list.appendChild(doc);
  });
};

document.onkeydown = function(e) {
  if (keybind_handler(e, "marked_document_picker")) {
    toggle_picker();
    current_action.innerText = "Mark Document";
    input.onkeydown = (e) => mark_document_bind(e);
  }
  if (keybind_handler(e, "current_document_picker")) {
    toggle_picker();
    current_action.innerText = "Set Current Document";
    input.onkeydown = (e) => load_document_bind(e);
  }
}

document.getElementById("marked_document").onclick = function() {
  toggle_picker();
  current_action.innerText = "Mark Document";
  input.onkeydown = (e) => mark_document_bind(e);
};

if (document.body.id === "view") {
  document.getElementById("current_document").onclick = function() {
    toggle_picker();
    current_action.innerText = "Set Current Document";
    input.onkeydown = (e) => load_document_bind(e);
  };
}

const load_document_bind = (e) => {
  if (e.key === "Enter") {
    if (results.length > 0 && Array.isArray(results)) {
      const document_name = results[0].item;
      document.getElementById("current_document").innerText = document_name;
      localStorage['current_document'] = document_name;
      Toastify({
        text: "Set current document to: " + document_name,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast();
      if (document.body.id === "view") {
        update_view(document_name);
      }
    } else {
      Toastify({
        text: "No documents found",
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast();
    }
    toggle_picker();
    input.value = "";
  }
}

const mark_document_bind = (e) => {
  if (e.key === "Enter") {
    if (results.length > 0 && Array.isArray(results)) {
      const document_name = results[0].item;
      invoke("mark_document", { documentName: document_name });
      document.getElementById("marked_document").innerText = document_name;
      Toastify({
        text: "Set marked document to: " + document_name,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast();
    } else {
      Toastify({
        text: "No documents found",
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast();
    }
    toggle_picker();
    input.value = "";
  }
}

let snippets = [];
const update_view = async (search_document) => {
  const marked_document = await invoke('fetch_marked_document')
    .catch((error) => console.log("Error caught:" + error));

  if (marked_document === null) {
    document.getElementById('marked_document').innerText = "None";
  } else {
    document.getElementById('marked_document').innerText = marked_document.document_name;
  }

  if (localStorage['current_document'] === null || localStorage['current_document'] === undefined) {
    localStorage['current_document'] = "None";
  }

  document.getElementById('current_document').innerText = localStorage['current_document'];

  if (localStorage['current_document'] === "None") {
    return
  }

  snippets = [];
  const snippet_container = document.getElementById('snippet');
  snippet_container.innerHTML = "";

  let document_name = "None";
  console.log(search_document);
  const viewed_document = await invoke('load_document', { documentName: search_document })
    .catch((error) => console.log("Error caught:" + error));

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

  document.getElementById("document_name").innerText = document_name;

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
  fuse.remove((item => {
    item.id === parseInt(id)
  }));
  edit_card.id = id;
  edit_card.value = snippet.raw;
  edit_card.oninput = () => {
    edit_card.style.height = "";
    edit_card.style.height = edit_card.scrollHeight + "px";
  }

  view_card.replaceWith(edit_card);
  edit_card.focus();
  toggle_overlay();

  document.getElementById("update_snippet").onclick = () => saveSnippet(edit_card, id);
  document.getElementById("delete_snippet").onclick = () => deleteSnippet(edit_card, id);
  document.getElementById("move_snippet").onclick = () => moveSnippet(edit_card, id);

  edit_card.onkeydown = (e) => {
    if (keybind_handler(e, "update_selected_snippet")) {
      saveSnippet(edit_card, id);
    }
    if (keybind_handler(e, "delete_selected_snippet")) {
      deleteSnippet(edit_card, id);
    }
    if (keybind_handler(e, "move_selected_snippet")) {
      moveSnippet(edit_card, id);
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
  toggle_overlay();
}

const deleteSnippet = (edit_card, id) => {
  snippets.pop(i => i.snippet_id === parseInt(id));
  invoke('delete_snippet', { snippetId: parseInt(id) });
  edit_card.remove();
  Toastify({
    text: "Deleted selected snippet",
    stopOnFocus: true,
    gravity: "bottom",
    position: "center"
  }).showToast();
  toggle_overlay();
}

const moveSnippet = (edit_card, id) => {
  toggle_picker();
  current_action.innerText = "Move Selected Snippet";
  input.onkeydown = (e) => move_document_bind(e, id, edit_card);
}

export const move_document_bind = (e, id, edit_card) => {
  if (e.key === "Enter") {
    if (results.length > 0 && Array.isArray(results)) {
      snippets.pop(i => i.snippet_id === parseInt(id));
      const document_name = results[0].item;
      invoke('move_snippet', { snippetId: parseInt(id), documentName: document_name })
        .then(() =>
          Toastify({
            text: "Moved snippet successfully to document: " + document_name,
            stopOnFocus: true,
            gravity: "bottom",
            position: "center"
          }).showToast()
        )
        .catch((error) =>
          Toastify({
            text: "Failed to move snippet due to: " + error,
            stopOnFocus: true,
            gravity: "bottom",
            position: "center"
          }).showToast()
        );
    } else {
      Toastify({
        text: "No results found",
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast();
    }
    edit_card.remove();
    toggle_picker();
    toggle_overlay();

    document.getElementById("document_input").value = "";
  }
}

const deleteDocument = () => {
  toggle_picker();
  current_action.innerText = "Delete Document";
  input.onkeydown = (e) => delete_document_bind(e);
}

const delete_document_bind = (e) => {
  if (e.key === "Enter") {
    if (results.length > 0 && Array.isArray(results)) {
      const document_name = results[0].item;
      invoke("delete_document", { documentName: document_name });
      if (document.body.id === "view" && localStorage['current_document'] === document_name) {
        document.getElementById("snippet").innerHTML = "";
        document.getElementById("document_name").innerText = "None";
        document.getElementById("current_document").innerText = "None";
        localStorage['current_document'] = "None";
      }
      results = results.filter(i => i.item !== document_name);
      fuse.remove((item => {
        item.title === document_name;
      }));
      Toastify({
        text: "Moved snippet successfully to document: " + document_name,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
    } else {
      Toastify({
        text: "No results found",
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast();
    }
    toggle_picker();

    document.getElementById("document_input").value = "";
  }
}

const deleteCurrentDocument = () => {
  let document_name = localStorage['current_document'];
  if (document_name !== "None") {
    invoke("delete_document", { documentName: document_name });
    if (document.body.id === "view") {
      document.getElementById("snippet").innerHTML = "";
      document.getElementById("document_name").innerText = "None";
      document.getElementById("current_document").innerText = "None";
    }
    results = results.filter(i => i.item !== document_name);
    console.log(fuse);
    console.log(document_name);
    fuse.remove((item => {
      item === document_name;
    }));
    localStorage['current_document'] = "None";
    Toastify({
      text: "Deleted current document: " + document_name,
      stopOnFocus: true,
      gravity: "bottom",
      position: "center"
    }).showToast()
  } else {
    Toastify({
      text: "No current document found, no documents deleted",
      stopOnFocus: true,
      gravity: "bottom",
      position: "center"
    }).showToast()
  }

  document.getElementById("document_input").value = "";
}
