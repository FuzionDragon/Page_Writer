import { marked } from 'marked';
import { invoke } from '@tauri-apps/api/core';
import Fuse from "fuse.js";

const first_element = document.getElementById('leftnav');
const picker_button = document.getElementById('toggle-picker');

const picker = document.createElement('div');
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

let results = [];
let fuse;

const toggle_picker = async () => {
  const document_picker = document.getElementById("document-picker");
  if (document_picker.style.display === "block") {
    document_picker.style.display = "none";
  } else {
    document_picker.style.display = "block";
    input.focus();
  }
}

document.addEventListener('DOMContentLoaded', async () => {
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
  results = fuse.search(query, {
    limit: 5,
  });

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

document.getElementById("marked_document").onclick = function() {
  toggle_picker();
};

input.onkeydown = function(e) {
  if (e.key === "Enter") {
    let document_name = "None";
    if (results.length > 0 && Array.isArray(results)) {
      console.log("Results found");
      document_name = results[0].item;
    } else {
      console.log("No results found");
    }

    toggle_picker();
    invoke("mark_document", { documentName: document_name }).then(() => {
      if (document.body.id === "view") {
        console.log("Updating view");
        update_view();
      }
    });
    document.getElementById("marked_document").innerText = document_name;
    input.value = "";
  }
}

const update_view = async () => {
  let snippet_container = document.getElementById('snippet');
  snippet_container.innerHTML = "";
  let document_name = "None";

  const marked_document = await invoke('fetch_marked_document')
    .catch((error) => console.log("Error caught:" + error));
  console.log(marked_document);

  const snippets = [];
  if (marked_document !== null) {
    document_name = marked_document.document_name;
    marked_document.snippets.forEach(snippet =>
      snippets.push({
        raw: snippet,
        markdown: marked.parse(snippet)
      })
    );
  }

  const document_title = document.createElement('h1');
  document_title.innerText = document_name;

  snippet_container.appendChild(document_title);

  snippets.forEach(snippet => {
    const card = document.createElement('div');
    //    card.className = "card";
    card.innerHTML = snippet.markdown;
    snippet_container.appendChild(card);
  })
}
