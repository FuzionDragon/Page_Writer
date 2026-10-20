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
syncButton.onclick = () => toggleTimout(syncButton);

const recoverButton = document.createElement('button');
recoverButton.innerText = "Recover From Backup";
recoverButton.id = "recover_notes";
recoverButton.onclick = () => toggleTimout(recoverButton);

const manualSaveButton = document.createElement('button');
manualSaveButton.innerText = "Manual Backup";
manualSaveButton.id = "manual_backup";
manualSaveButton.onclick = () => toggleTimout(manualSaveButton);

menu.appendChild(title);
menu.appendChild(syncButton);
menu.appendChild(recoverButton);
menu.appendChild(manualSaveButton);

document.getElementById("navbar").appendChild(menu);

export const toggle_sync_menu = async () => {
  if (menu.style.display === "block") {
    menu.style.display = "none";
  } else {
    menu.style.display = "block";
  }
}

const toggleTimout = (btn) => {
  const innerText = btn.innerText;
  console.log(innerText);

  switch (btn.id) {
    case "sync_notes":
      btn.onclick = () => sync_notes();
      break;
    case "recover_notes":
      btn.onclick = () => recover_notes();
      break;
    case "manual_backup":
      btn.onclick = () => manual_backup();
      break;
    default:
      console.log("Invalid button");
      return;
  }

  btn.innerText = "Confirm Operation?"
  setTimeout(function() {
    btn.onclick = () => toggleTimout(btn);
    btn.innerText = innerText;

    console.log("Timeout, reseting button");
  }, 5000);
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

  syncButton.onclick = () => toggleTimout(self);
  syncButton.innerText = "Sync Data Over Local Network";
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

  recoverButton.onclick = () => toggleTimout(self);
  recoverButton.innerText = "Recover From Backup";
}

export const manual_backup = function() {
  console.log("Manual backup");
  manualSaveButton.onclick = () => toggleTimout(self);
  manualSaveButton.innerText = "Manual Backup";
}
