import Toastify from 'toastify-js'
import { invoke } from '@tauri-apps/api/core';
import { keybind_handler } from './config';

const snippet_input = document.getElementById('snippet-input');
const title_input = document.getElementById('title-input');

document.addEventListener('DOMContentLoaded', async () => {
  title_input.focus();
});

const submit = function() {
  invoke('submit', { snippet: snippet_input.value, title: title_input.value })
    .then(() => {
      Toastify({
        text: "Successfully submitted snippet",
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
    })
    .catch((error) =>
      Toastify({
        text: "Error submitting snippet" + error,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
    );

  snippet_input.value = "";
  title_input.value = "";
}

document.getElementById("submit-snippet").onclick = () => submit();

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
  //  Toastify({
  //    text: "Keydown: " + e.key,
  //    stopOnFocus: true,
  //    gravity: "bottom",
  //    position: "center"
  //  }).showToast()

  if (keybind_handler(e, "submit_snippet")) {
    submit();
  }
  if (keybind_handler(e, "switch_menu")) {
    window.location.href = "./view.html";
  }
  if (keybind_handler(e, "export_notes")) {
    invoke("export_all_documents")
      .then((path) => {
        Toastify({
          text: "Successfully exported all notes to: " + path,
          stopOnFocus: true,
          gravity: "bottom",
          position: "center"
        }).showToast()
      })
      .catch((error) =>
        Toastify({
          text: "Error submitting snippet" + error,
          stopOnFocus: true,
          gravity: "bottom",
          position: "center"
        }).showToast()
      );
  }
}
