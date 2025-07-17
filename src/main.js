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
  if (e.ctrlKey && e.key === "t") {
    window.location.href = "./pages/view.html";
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
