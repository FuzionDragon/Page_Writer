# Page Writer

A powerful and unique note-taking application for spontaneous note takers and power users.

Notes are treated as snippets of text that belongs to a collection of other related snippets, making up a document. The application allows for the manual and automatic means of assigning a snippet to a document.

They are currently pieces of Markdown which is stored in a SQLite database, **however there are plans for exporting and importing actual Markdown files**.

### Core Features

- Minimalistic interface with two pages, one for submitting snippets and another for viewing your snippets in the form of a rendered document.

- Fuzzy finder used for selecting, marking, and deleting your documents, powered by Fuse.js.

- The user has the ability to quickly and easily assign notes to a specific document by simply giving a name of an existing document, if it doesn't exist then a document with the corresponding name will be created.

- Allows the user to mark a document, making all untitled snippets automatically assign themselves to the marked document.

- Interactive document view page with the ability to add, update, delete, and move individual snippets.

- Local network data syncing of notes between two devices.

- Android and Linux compatibility (Windows planned for the future).

- Built using Tauri with vanilla JavaScript, SCSS for the frontend, and Rust as a backend.

### Keybinds

Keybinds are a central part of Page Writers usage and allows for better efficiency when navigating the application.

**Currently, some keybinds are hardcoded and cannot be changed as of writing, this is planned in the future alongside a configuration file and a potential settings menu**

Submitting a snippet on the input menu : Control + Enter

Switch between input and view menus : Control + T

Toggle picker for current document : Control + E

Toggle picker for marking a document : Control + R

Toggle picker for deleting a document : Control + D

Delete current document on input menu : Control + Delete

Toggle add snippet on view menu : Control + Q

Add new snippet on the view menu : Control + Enter

Delete current snippet : Control + Delete

Delete selected snippet : Delete

Move selected snippet : Control + F

Update selected snippet : Control + Enter

Scroll top in view menu : G

Scroll bottom in view menu : Control + G

Toggle sync menu : Control + Shift + M

Toggle shortcuts menu : Control + H

### TODO

- Windows support (no plans for iOS or MAC as of writing).

- Settings menu.

- Document explorer?

- Other QoL additions that I am not thinking of.

- Potentially multiple databases?

- Spell Checker?

- Markdown imported both file and directory based?
