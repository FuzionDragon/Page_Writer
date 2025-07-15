import { invoke } from '@tauri-apps/api/core';

const submit = function() {
  let snippet = document.getElementById('snippet_input');
  invoke('submit', { snippet: snippet.value })
    .catch((error) => console.log(error));
  snippet.value = "";
}

window.onkeyup = function(e) {
  if (e.ctrlKey && e.key === 'Enter') {
    submit();
  }
}
