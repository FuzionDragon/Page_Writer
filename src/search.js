import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";

const first_element = document.getElementById('leftnav');
const picker_button = document.getElementById('toggle-picker');
let document_list;
let input;
let picker;
let fuse;

const toggle_picker = async () => {
  const document_picker = document.getElementById("document-picker");
  if (document_picker.style.display === "block") {
    document_picker.style.display = "none";
  } else {
    document_picker.style.display = "block";
  }
}

document.addEventListener('DOMContentLoaded', async () => {
  picker = document.createElement('div');
  picker.id = "document-picker";
  picker.className = "overlay-document-picker";

  input = document.createElement('input');
  input.type = "text";
  input.placeholder = "Searching documents... ";
  input.id = "document_input";

  document_list = document.createElement('ul');
  document_list.id = "document_list";

  picker.appendChild(input);
  picker.appendChild(document_list);

  document.body.insertBefore(picker, first_element);

  const corpus = await invoke('load_snippets')
    .catch((error) => console.log("error caught:" + error));

  console.log("Object: " + corpus);
  fuse = new Fuse(Object.keys(corpus), {
    keys: ['title'],
    threshold: 0.4
  });
})

document.oninput = function(e) {
  document_list.innerHTML = '';
  const query = e.target.value;
  const results = fuse.search(query);

  results.forEach(result => {
    console.log(result.item);
    const doc = document.createElement('li');
    doc.innerText = result.item;
    document_list.appendChild(doc);
  });
};

document.onkeydown = function(e) {
  if (e.ctrlKey && e.key === "e") {
    toggle_picker();
  }
}

picker_button.onclick = function() {
  toggle_picker();
};

