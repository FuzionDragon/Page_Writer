import { invoke } from "@tauri-apps/api/core";

document.addEventListener("DOMContentLoaded", async () => {
  console.log("running config script");
  if (localStorage["keybinds"] === undefined) {
    invoke('load_config')
      .catch((error) => console.log("Error found: " + error));
  }
})
