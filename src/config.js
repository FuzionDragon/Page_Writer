import { invoke } from "@tauri-apps/api/core";

document.addEventListener("DOMContentLoaded", async () => {
  console.log("running config script");
  if (localStorage["keybinds"] === undefined) {
    invoke('load_keybindings')
      .then((keybindings) => {
        localStorage['keybinds'] = keybindings;
      })
      .catch((error) => console.log("Error found: " + error));
  }
})

const parse_keybind = (keybind) => {
  const parts = keybind.split('+').map(part => part.trim());
  const modifiers = parts.slice(0, -1);
  const key = parts[parts.length - 1];

  return {
    ctrl: modifiers.includes('Ctrl'),
    shift: modifiers.includes('Shift'),
    alt: modifiers.includes('Alt'),
    meta: modifiers.includes('Meta'),
    key: key
  };
}

export const keybind_handler = (e, command) => {
  const keybinds = localStorage["keybinds"];
  if ((keybinds[command] === undefined) || (keybinds[command] === null)) {
    console.log("keybind not found");

    switch (command) {
      case "switch_menu":
        return (e.ctrlKey && e.key === "t")
      case "submit_snippet":
        return (e.ctrlKey && e.key === "Enter")
      case "current_document_picker":
        console.log("current_document_picker");
        return (e.ctrlKey && e.key === "e")
      case "marked_document_picker":
        console.log("marked_document_picker");
        return (e.ctrlKey && e.key === "r")
      case "delete_document_picker":
        console.log("delete_document_picker");
        return (e.ctrlKey && e.key === "d")
      case "delete_current_document":
        console.log("delete_current_document");
        return (e.ctrlKey && e.key === "Del")
      case "move_selected_snippet":
        console.log("move_selected_snippet");
        return (e.ctrlKey && e.key === "f")
      case "delete_selected_snippet":
        console.log("delete_selected_snippet");
        return (e.key === "Del")
      case "update_selected_snippet":
        console.log("update_selected_snippet");
        return (e.ctrlKey && e.key === "Enter")
      default:
        console.log("Command not found");
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
