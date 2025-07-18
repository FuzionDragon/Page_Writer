import { invoke } from '@tauri-apps/api/core';

const snippet_input = document.getElementById('snippet-input');
const title_input = document.getElementById('title-input');

const submit = function() {
  invoke('submit', { snippet: snippet_input.value, title: title_input.value })
    .catch((error) => console.log(error));
  snippet_input.value = "";
  title_input.value = "";
}

title_input.onkeydown = function(e) {
  if ((e.key === 'ArrowDown' || e.key === 'ArrowRight') && title_input.selectionEnd === title_input.value.length) {
    e.preventDefault();
    snippet_input.focus();
  }
  if (e.key === 'Enter') {
    e.preventDefault();
    snippet_input.focus();
  }
}

snippet_input.onkeydown = function(e) {
  if ((e.key === 'ArrowUp' || e.key === 'ArrowLeft') && snippet_input.selectionStart === 0) {
    e.preventDefault();
    title_input.focus();
    title_input.setSelectionRange(title_input.value.length, title_input.value.length);
  }
}

window.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "Enter") {
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
