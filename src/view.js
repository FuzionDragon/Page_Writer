import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

export let snippets = [];

document.addEventListener('DOMContentLoaded', async () => {
  const marked_document = await invoke('fetch_marked_document')
    .catch((error) => console.log("Error caught:" + error));

  document.getElementById('marked_document').innerText = marked_document.document_name;

  if (marked_document.document_name === "None") {
    document.getElementById('rightnav').hidden = true;
  } else {
    document.getElementById('rightnav').hidden = false;
  }
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

  const document_title = document.createElement('h1');
  document_title.innerText = document_name;

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

  //  edit_card.onblur = () => {
  //    saveSnippet(edit_card, view_card.id);
  //    toggle_overlay();
  //  }
  //
  document.getElementById("update_snippet").onclick = () => saveSnippet(edit_card, id);
  document.getElementById("delete_snippet").onclick = () => deleteSnippet(edit_card, id);
  document.getElementById("move_snippet").onclick = () => moveSnippet(edit_card, id);

  edit_card.onkeydown = (e) => {
    if (e.ctrlKey && e.key === "Enter") {
      saveSnippet(edit_card, id);
      toggle_overlay();
    }
  }
}

const saveSnippet = (edit_card, id) => {
  console.log("Updating snippet");
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

const deleteSnippet = (edit_card, id) => {
  console.log("Deleting snippet");
  snippets.pop(i => i.snippet_id === parseInt(id));
  invoke('delete_snippet', { snippetId: parseInt(id) });
  edit_card.remove();
  toggle_overlay();
}

const moveSnippet = (edit_card, id) => {
  console.log("Moving snippet");
  toggle_picker();
  input.onkeydown = (e, id) => move_document_bind(e, id);
}

const move_document_bind = (e) => {
  if (e.key === "Enter") {
    let document_name = "None";
    if (results.length > 0 && Array.isArray(results)) {
      console.log("Results found");
      snippets.pop(i => i.snippet_id === parseInt(id));
      document_name = results[0].item;
    } else {
      console.log("No results found");
    }
    toggle_picker();

    invoke('move_snippet', { snippetId: parseInt(id), documentName: document_name });
    input.value = "";
  }
}

