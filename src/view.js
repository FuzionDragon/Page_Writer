import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

export let snippets = [];

document.addEventListener('DOMContentLoaded', async () => {
  renderView(localStorage['current_document']);
});

const renderView = async (search_document) => {
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

  document.getElementById('marked_document').innerText = document_name;

  const document_title = document.createElement('h1');
  document_title.innerText = viewed_document.document_name;

  if (document_name === "None") {
    document.getElementById('rightnav').hidden = true;
  } else {
    document.getElementById('rightnav').hidden = false;
  }

  if (localStorage['current_document'] === null) {
    localStorage['current_document'] = "None";
  }

  document.getElementById('current_document').innerText = localStorage['current_document'];

  let snippet_container = document.getElementById('snippet');
  snippet_container.appendChild(document_title);

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

  edit_card.onblur = () => {
    saveSnippet(edit_card, view_card.id);
    toggle_overlay();
  }

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

window.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "t") {
    window.location.href = "../index.html";
  }
}
