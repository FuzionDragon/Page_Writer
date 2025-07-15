import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

document.addEventListener('DOMContentLoaded', async () => {
  let snippet_container = document.getElementById('snippet');

  const corpus = await invoke('load_snippets')
    .catch((error) => console.log("error caught:" + error));

  console.log("From app: " + corpus);
  for (const [key, value] of Object.entries(corpus)) {
    const card = document.createElement('div');
    card.className = "card";
    const title = document.createElement('h2');
    title.textContent = key;
    const content = document.createElement('p')
    content.textContent = value;
    card.appendChild(title);
    card.appendChild(content);
    snippet_container.appendChild(card);
  }
});
