import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';
import { move_document_bind, toggle_picker } from './search';
import { keybind_handler } from './config';
import Toastify from 'toastify-js'

let snippets = [];
document.addEventListener('DOMContentLoaded', async () => {
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

  if (localStorage['current_document'] !== "None") {
    renderView(localStorage['current_document']);
  }
});

const renderView = async (search_document) => {
  snippets = [];
  // requires the cache to have the document id
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

  const snippet_container = document.getElementById('snippet');
  document.getElementById("document_name").innerText = document_name;

  for (const snippet of snippets.entries()) {
    const view_card = document.createElement('div');
    view_card.innerHTML = snippet[1].markdown;
    view_card.id = snippet[1].snippet_id;
    view_card.onclick = () => editSnippet(view_card);

    snippet_container.appendChild(view_card);
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

const editSnippet = (view_card) => {
  const edit_card = document.createElement('textarea');
  const id = view_card.id;
  const snippet = snippets.find(i => i.snippet_id === parseInt(id));
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
  Toastify({
    text: "Updated snippet",
    stopOnFocus: true,
    gravity: "bottom",
    position: "center"
  }).showToast()
  toggle_overlay();
}

window.onkeydown = function(e) {
  if (keybind_handler(e, "switch_menu")) {
    window.location.href = "../index.html";
  }
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
  }).showToast()
  toggle_overlay();
}

const moveSnippet = (edit_card, id) => {
  toggle_picker();
  document.getElementById("document_input").onkeydown = (e) => move_document_bind(e, id, edit_card);
}
