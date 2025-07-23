import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

let snippets = [];

document.addEventListener('DOMContentLoaded', async () => {
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
  console.log(snippets);

  document.getElementById('marked_document').innerText = document_name;

  const document_title = document.createElement('h1');
  document_title.innerText = marked_document.document_name;

  let snippet_container = document.getElementById('snippet');
  snippet_container.appendChild(document_title);

  for (let id = 0; id < snippets.length; id++) {
    let snippet = snippets[id];
    console.log(id);
    const view_card = document.createElement('div');
    view_card.innerHTML = snippet.markdown;
    view_card.id = id;
    view_card.onclick = () => editSnippet(view_card);
    console.log(id);

    snippet_container.appendChild(view_card);
  }
});

const editSnippet = (view_card) => {
  const edit_card = document.createElement('textarea');
  const id = view_card.id;
  console.log(id);
  edit_card.id = id;
  console.log(snippets[id]);
  edit_card.value = snippets[id].raw;

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
  snippets[id].raw = edit_card.value;
  snippets[id].markdown = content;

  const view_card = document.createElement('div');
  view_card.innerHTML = content;
  view_card.id = id;
  view_card.onclick = () => editSnippet(view_card);
  edit_card.replaceWith(view_card);
}

window.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "t") {
    window.location.href = "../index.html";
  }
}
