import { invoke } from '@tauri-apps/api/core';
import Toastify from 'toastify-js'

const default_bindings = {
  "switch_menu": "Control+t",
  "submit_snippet": "Control+Enter",
  "search_document": "Control+f",
  "current_document_picker": "Control+e",
  "marked_document_picker": "Control+r",
  "delete_document_picker": "Control+d",
  "delete_current_document": "Control+Delete",
  "delete_selected_snippet": "Delete",
  "move_selected_snippet": "Control+f",
  "update_selected_snippet": "Control+Enter",
  "toggle_add_snippet": "Control+q",
  "add_snippet": "Control+Enter",
  "toggle_shortcuts_menu": "Control+h",
  "scroll_top": "g",
  "scroll_bottom": "Control+g",
  "sync_notes": "Control+Shift+m"
};

document.addEventListener("DOMContentLoaded", async () => {
  console.log("running config script");
  console.log("Setting keybinds");
  invoke('load_keybindings')
    .then((keybindings) => {
      if (keybindings !== null && keybinds !== localStorage["keybinds"]) {
        localStorage['keybinds'] = JSON.stringify(keybindings);
      } else if (default_bindings !== localStorage["keybinds"]) {
        localStorage["keybinds"] = JSON.stringify(default_bindings);
      }
    })
    .catch((error) => {
      console.log("Error found: " + error);
      Toastify({
        text: "Error found:" + error,
        stopOnFocus: true,
        gravity: "bottom",
        position: "center"
      }).showToast();
    })

  render_shortcuts_menu();
})

const render_shortcuts_menu = () => {
  const menu = document.createElement('div');
  menu.id = "shortcuts-menu";
  menu.className = "container";
  menu.style.display = "none";
  const shortcuts_title = document.createElement('h2');
  shortcuts_title.innerText = "Shortcuts";
  shortcuts_title.style.width = "100%";
  menu.appendChild(shortcuts_title);
  document.getElementById("navbar").appendChild(menu);

  const data = localStorage.getItem("keybinds");
  let keybinds = data ? JSON.parse(data) : {};

  for (const [key, value] of Object.entries(keybinds)) {
    const card = document.createElement("div");
    card.className = "card";
    const title = document.createElement("h3");
    title.innerText = key;
    const binding = document.createElement("p");
    binding.innerText = value;
    card.appendChild(title);
    card.appendChild(binding);
    menu.appendChild(card)
  }
}

export const toggle_shortcuts_menu = () => {
  console.log("toggle_shortcuts_menu")
  const menu = document.getElementById("shortcuts-menu");
  if (menu.style.display === "flex") {
    menu.style.display = "none";
  } else {
    menu.style.display = "flex";
  }
}

const parse_keybind = (keybind) => {
  const keys = keybind.split('+').map(key => key.trim().toLowerCase());
  const modifiers = keys.slice(0, -1);
  const key = keys[keys.length - 1];

  return {
    ctrl: modifiers.includes('control'),
    shift: modifiers.includes('shift'),
    alt: modifiers.includes('alt'),
    meta: modifiers.includes('meta'),
    key: key
  };
}

export const keybind_handler = (e, command) => {
  const data = localStorage.getItem("keybinds");
  let keybinds = data ? JSON.parse(data) : {};

  const parsed_keybind = parse_keybind(keybinds[command]);

  return (
    (parsed_keybind.ctrl === e.ctrlKey) &&
    (parsed_keybind.shift === e.shiftKey) &&
    (parsed_keybind.alt === e.altKey) &&
    (parsed_keybind.meta === e.metaKey) &&
    (parsed_keybind.key.toUpperCase() === e.key.toUpperCase())
  );
}
