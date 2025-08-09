import Toastify from 'toastify-js'
import { invoke } from '@tauri-apps/api/core';
import { keybind_handler } from './config';

const snippet_input = document.getElementById('snippet-input');
const title_input = document.getElementById('title-input');

// only for testing
function getBrowser() {
  if (navigator.userAgent.indexOf("Chrome") != -1) {
    return "Chrome";
  } else if (navigator.userAgent.indexOf("Opera") != -1) {
    return "Opera";
  } else if (navigator.userAgent.indexOf("MSIE") != -1) {
    return "IE";
  } else if (navigator.userAgent.indexOf("Firefox") != -1) {
    return "Firefox";
  } else {
    return "unknown";
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  Toastify({
    text: getBrowser(),
    stopOnFocus: true,
    gravity: "bottom",
    position: "center"
  }).showToast()

  invoke('get_config_path')
    .then((path) =>
      Toastify({
        text: path,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
    )
    .catch((error) =>
      Toastify({
        text: error,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
    )

  const marked_document = await invoke('fetch_marked_document')
    .catch((error) => console.log("Error caught:" + error));

  if (marked_document === null) {
    document.getElementById('marked_document').innerText = "None";
  } else {
    document.getElementById('marked_document').innerText = marked_document.document_name;
  }
})

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
  invoke('get_android_path_tauri')
    .then((path) => {
      Toastify({
        text: "Android path: " + path,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
    })
    .catch((error) =>
      Toastify({
        text: "Error getting path: " + error,
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
  if (keybind_handler(e, "submit_snippet")) {
    submit();
  }
  if (keybind_handler(e, "switch_menu")) {
    window.location.href = "./view.html";
  }
}
