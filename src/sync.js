import Toastify from 'toastify-js'
import { invoke } from '@tauri-apps/api/core';
import { rebuildFuse } from './search';

const picker_container = document.createElement('div');
picker_container.id = "picker_container";

const menu = document.createElement('div');
menu.className = "overlay-sync-menu";

const title = document.createElement('h3');
title.innerText = "Sync Menu";

const syncButton = document.createElement('button');
syncButton.innerText = "Sync Data Over Local Network";
syncButton.id = "sync_notes";

const recoverButton = document.createElement('button');
recoverButton.innerText = "Recover From Backup";
recoverButton.id = "recover_notes";

menu.appendChild(title);
menu.appendChild(syncButton);
menu.appendChild(recoverButton);

document.getElementById("navbar").appendChild(menu);

document.getElementById("sync_notes").onclick = () => sync_notes();
document.getElementById("sync_notes").onclick = () => recover_notes();

export const toggle_sync_menu = async () => {
  if (menu.style.display === "block") {
    menu.style.display = "none";
  } else {
    menu.style.display = "block";
  }
}

export const recover_notes = function() {
  Toastify({
    text: "Overriding current data with backup",
    stopOnFocus: true,
    gravity: "bottom",
    position: "center"
  }).showToast();
  invoke("load_backup")
    .then((response) => {
      Toastify({
        text: "Response: " + response,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
      console.log(response);
      Toastify({
        text: "No error with load_backup",
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()

      rebuildFuse();
    })
    .catch((error) => {
      Toastify({
        text: "Error: " + error,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
      console.log(error);
    }
    );
}

export const sync_notes = function() {
  Toastify({
    text: "Searching local network for devices running Page Writer...",
    stopOnFocus: true,
    gravity: "bottom",
    position: "center"
  }).showToast();
  // needs to be replaced by different function carrying out both
  invoke("sync_data")
    .then((response) => {
      Toastify({
        text: "Response: " + response,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
      console.log(response);
      Toastify({
        text: "No error with sync_data",
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()

      rebuildFuse();
    })
    .catch((error) => {
      Toastify({
        text: "Error: " + error,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast()
      console.log(error);
    }
    );
}
