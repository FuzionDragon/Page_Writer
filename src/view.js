import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

document.addEventListener('DOMContentLoaded', async () => {
  const marked_document = await invoke('fetch_marked_document')
    .catch((error) => console.log("Error caught:" + error));

  const snippets = [];
  marked_document.snippets.forEach(snippet =>
    snippets.push({
      raw: snippet,
      markdown: marked.parse(snippet)
    })
  );

  let marked_document_label = document.getElementById('marked_document');
  marked_document_label.innerText = marked_document.document_name;

  const document_title = document.createElement('h1');
  document_title.innerText = marked_document.document_name;

  let snippet_container = document.getElementById('snippet');
  document.body.appendChild(document_title);

  snippets.forEach(snippet => {
    const card = document.createElement('div');
    card.className = "card";
    card.innerHTML = snippet.markdown;
    document.body.appendChild(card);
  })
});

window.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "t") {
    window.location.href = "../index.html";
  }
}
