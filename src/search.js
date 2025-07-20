import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";

const first_element = document.getElementById('leftnav');
const picker_button = document.getElementById('toggle-picker');
let picker;

const toggle_picker = async () => {
  const document_picker = document.getElementById("document-picker");
  if (document_picker.style.display === "block") {
    document_picker.style.display = "none";
  } else {
    document_picker.style.display = "block";
    const corpus = await invoke('load_snippets')
      .catch((error) => console.log("error caught:" + error));

    const fuse = new Fuse(Object.keys(corpus), {
      keys: ['title'],
      threshold: 0.4
    });
  }
}

document.addEventListener('DOMContentLoaded', async () => {
  picker = document.createElement('div');
  picker.id = "document-picker";
  picker.className = "overlay-document-picker";

  const input = document.createElement('input');
  input.type = "text";
  input.placeholder = "Searching documents... ";
  input.id = "document_input";

  const document_list = document.createElement('ul');
  document_list.id = "document_list";

  picker.appendChild(input);
  picker.appendChild(document_list);

  document.body.insertBefore(picker, first_element);
})

document.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "e") {
    toggle_picker();
  }
}

picker_button.onclick = function() {
  toggle_picker();
};

