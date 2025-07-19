import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";

const first_element = document.getElementById('leftnav');
const picker_button = document.getElementById('toggle-picker');
let picker;
let search;

const toggle_picker = () => {
  const document_picker = document.getElementById("document-picker");
  if (document_picker.style.display === "block") {
    document_picker.style.display = "none";
  } else {
    document_picker.style.display = "block";
  }
}

picker_button.onclick = function() {
  toggle_picker();
};

const picker_items = async () => {
  const corpus = await invoke('load_snippets')
    .catch((error) => console.log("error caught:" + error));
}

document.addEventListener('DOMContentLoaded', async () => {
  picker = document.createElement('div');
  picker.id = "document-picker";
  picker.className = "overlay-document-picker";

  search = document.createElement('div');
  search.className = "card";

  const title = document.createElement('h1');
  title.innerText = "This is a file picker";

  search.appendChild(title);
  picker.appendChild(search);

  document.body.insertBefore(picker, first_element);
})

//const fuse = new Fuse()
document.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "e") {
    toggle_picker();
  }
}
