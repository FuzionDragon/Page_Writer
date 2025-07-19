import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";

const first_element = document.getElementById('leftnav');
const picker_button = document.getElementById('toggle-picker');
let picker;
let search;

const toggle_picker = async () => {
  const document_picker = document.getElementById("document-picker");
  if (document_picker.style.display === "block") {
    document_picker.style.display = "none";
  } else {
    document_picker.style.display = "block";
    const options = {
      keys: ["title"]
    };
    const corpus = await invoke('load_snippets')
      .catch((error) => console.log("error caught:" + error));

    const fuse = new Fuse(Object.keys(corpus), options);
  }
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

document.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "e") {
    toggle_picker();
  }
}

picker_button.onclick = function() {
  toggle_picker();
};

