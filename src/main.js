import Toastify from 'toastify-js'
import { invoke } from '@tauri-apps/api/core';
import { keybind_handler } from './config';

const snippet_input = document.getElementById('snippet-input');
const title_input = document.getElementById('title-input');

document.addEventListener("DOMContentLoaded", async () => {
  const marked_document = await invoke('fetch_marked_document')
    .catch((error) => console.log("Error caught:" + error));

  if (marked_document === null) {
    document.getElementById('marked_document').innerText = "None";
  } else {
    document.getElementById('marked_document').innerText = marked_document.document_name;
  }

  if (document.getElementById('marked_document').innerText === "None") {
    document.getElementById('rightnav').hidden = true;
  } else {
    document.getElementById('rightnav').hidden = false;
  }
})

const submit = function() {
  invoke('submit', { snippet: snippet_input.value, title: title_input.value })
    .then((document_id) => {
      if (document_id !== null) {
        localStorage['current_document_id'] = document_id;
      }
      Toastify({
        text: "Successfully submitted snippet",
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
    })
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

snippet_input.oninput = () => {
  snippet_input.style.height = "";
  snippet_input.style.height = snippet_input.scrollHeight + "px";
}

window.onkeydown = function(e) {
  if (keybind_handler(e, "submit_snippet")) {
    submit();
  }
  if (keybind_handler(e, "switch_menu")) {
    window.location.href = "./view.html";
  }
}
