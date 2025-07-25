import { invoke } from '@tauri-apps/api/core';

const snippet_input = document.getElementById('snippet-input');
const title_input = document.getElementById('title-input');

document.addEventListener("DOMContentLoaded", async () => {
  const marked_document = await invoke('fetch_marked_document')
    .catch((error) => console.log("Error caught:" + error));

  document.getElementById('marked_document').innerText = marked_document.document_name;

  if (marked_document.document_name === "None") {
    document.getElementById('rightnav').hidden = true;
  } else {
    document.getElementById('rightnav').hidden = false;
  }

  if (localStorage['current_document'] === null) {
    localStorage['current_document'] = "None";
  }

  document.getElementById('current_document').innerText = localStorage['current_document'];
})

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
}
