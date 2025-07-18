import { Fuse } from 'fuse.js';

let document_picker;

document.addEventListener('DOMContentLoaded', async () => {
  document_picker = document.createElement('div');
  document_picker.id = "document-picker";
  document_picker.className = "overlay-document-picker";
})
//    <div id="document-picker" class="overlay-document-picker">
//      <div class="card">
//        <h1>This is a file picker</h1>
//      </div>
//    </div>
const fuseOptions = {
  keys: [
    "title"
  ]
};

//const fuse = new Fuse()
