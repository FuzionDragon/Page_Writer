import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

document.addEventListener('DOMContentLoaded', async () => {
  let snippet_container = document.getElementById('snippet');

  const corpus = await invoke('load_snippets')
    .catch((error) => console.log("error caught:" + error));

  console.log("From app: " + corpus);
  const documents = Object.entries(corpus).map(([document_name, snippets]) => {
    const new_snippets = [];
    snippets.forEach(snippet => {
      new_snippets.push({
        raw: snippet,
        markdown: marked.parse(snippet)
      })
    })
    return {
      document_name,
      new_snippets
    }
  });

  documents.forEach(document_obj => {
    const card = document.createElement('div');
    card.className = "card";
    let title = document.createElement('h3');
    title.innerText = document_obj.document_name;
    card.appendChild(title);
    document_obj.new_snippets.forEach(snippet => {
      const snippet_item = document.createElement('div');
      snippet_item.innerHTML = snippet.markdown;
      card.appendChild(snippet_item);
    })
    snippet_container.appendChild(card);
  })
});

window.onkeyup = function(e) {
  if (e.ctrlKey && e.key === "t") {
    window.location.href = "../index.html";
  }
  if (e.ctrlKey && e.key === "e") {
    const document_picker = document.getElementById("document-picker");
    if (document_picker.style.display === "block") {
      document_picker.style.display = "none";
    } else {
      document_picker.style.display = "block";
    }
  }
}
