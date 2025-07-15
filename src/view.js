import { invoke } from '@tauri-apps/api/core';
import { marked } from 'marked';

document.addEventListener('DOMContentLoaded', async () => {
  let snippet = document.getElementById('snippet');

  const corpus = await invoke('load_snippets')
    .catch((error) => console.log("error caught:" + error));

  console.log("From app: " + corpus);
  for (const [key, value] of Object.entries(corpus)) {
    console.log(key + value);
  }
});
