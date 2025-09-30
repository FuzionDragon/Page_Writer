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
  "toggle_shortcuts_menu": "Control+h"
};

document.addEventListener("DOMContentLoaded", async () => {
  console.log("running config script");
  if (localStorage["keybinds"] === undefined || localStorage["keybinds"] === null) {
    console.log("Setting keybinds");
    invoke('load_keybindings')
      .then((keybindings) => {
        if (keybindings !== null) {
          localStorage['keybinds'] = JSON.stringify(keybindings);
          Toastify({
            text: "Keybinds loaded from config",
            stopOnFocus: true,
            gravity: "bottom",
            position: "center"
          }).showToast();
        } else {
          localStorage["keybinds"] = JSON.stringify(default_bindings);
          Toastify({
            text: "Keybinds loaded from default_bindings",
            stopOnFocus: true,
            gravity: "bottom",
            position: "center"
          }).showToast();
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

    Toastify({
      text: "Keybinds not found, caching based on config file",
      stopOnFocus: true,
      gravity: "bottom",
      position: "center"
    }).showToast();
  }

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
//  Toastify({
//    text: "Parsing keybind: " + keybind,
//    stopOnFocus: true,
//    gravity: "bottom",
//    position: "center"
//  }).showToast();

  const parts = keybind.split('+').map(part => part.trim());
  const modifiers = parts.slice(0, -1);
  const key = parts[parts.length - 1];

  return {
    ctrl: modifiers.includes('Control'),
    shift: modifiers.includes('Shift'),
    alt: modifiers.includes('Alt'),
    meta: modifiers.includes('Meta'),
    key: key
  };
}

export const keybind_handler = (e, command) => {
  const data = localStorage.getItem("keybinds");
  let keybinds = data ? JSON.parse(data) : {};
//  Toastify({
//    text: "handing command: " + command + "checking binding: " + e,
//    stopOnFocus: true,
//    gravity: "bottom",
//    position: "center"
//  }).showToast();
  if ((keybinds[command] === undefined) || (keybinds[command] === null)) {
    switch (command) {
      case "switch_menu":
        keybinds["switch_menu"] = default_bindings["switch_menu"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "t")
      case "submit_snippet":
        keybinds["submit_snippet"] = default_bindings["submit_snippet"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "Enter")
      case "search_document":
        keybinds["search_document"] = default_bindings["search_document"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "f")
      case "current_document_picker":
        keybinds["current_document_picker"] = default_bindings["current_document_picker"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "e")
      case "marked_document_picker":
        keybinds["marked_document_picker"] = default_bindings["marked_document_picker"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "r")
      case "delete_document_picker":
        keybinds["delete_document_picker"] = default_bindings["delete_document_picker"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "d")
      case "delete_current_document":
        keybinds["delete_current_document"] = default_bindings["delete_current_document"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "Delete")
      case "move_selected_snippet":
        keybinds["move_selected_snippet"] = default_bindings["move_selected_snippet"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "f")
      case "delete_selected_snippet":
        keybinds["delete_selected_snippet"] = default_bindings["delete_selected_snippet"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.key === "Delete")
      case "update_selected_snippet":
        keybinds["update_selected_snippet"] = default_bindings["update_selected_snippet"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "Enter")
      case "toggle_add_snippet":
        keybinds["toggle_add_snippet"] = default_bindings["toggle_add_snippet"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "q")
      case "add_snippet":
        keybinds["add_snippet"] = default_bindings["add_snippet"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "Enter")
      case "toggle_shortcuts_menu":
        console.log("Pressed toggle_shortcuts_menu")
        keybinds["toggle_shortcuts_menu"] = default_bindings["toggle_shortcuts_menu"];
        localStorage.setItem("keybinds", JSON.stringify(keybinds));
        return (e.ctrlKey && e.key === "h")
      default:
        break;
    }
  } else {
    const parsed_keybind = parse_keybind(keybinds[command]);

    return (
      (parsed_keybind.ctrl === e.ctrlKey) &&
      (parsed_keybind.shift === e.shiftKey) &&
      (parsed_keybind.alt === e.altKey) &&
      (parsed_keybind.meta === e.metaKey) &&
      (parsed_keybind.key.toUpperCase() === e.key.toUpperCase())
    );
  }
}
