# Page Writer

A powerful note taking application for spontanious note takers and power users.

Notes are treated as snippets of text that belongs to a collection of other related snippets, making up a document.

They are currently pieces of markdown which is stored in a Sqlite database, **however there are plans for exporting and importing actual markdown files**.

### Core Features

- Minimalistic interface with two pages, one for submitting snippets and another for viewing your snippets in the form of a rendered document.

- Fuzzy finder used for selecting, marking and deleting your documents, powered by Fuse.js.

- The user has the ability to quickly and easily merge notes with a specific document by simply giving a name of an existing document, if it doesn't exist then a document with the corresponding name will be created.

- Notes without an entered name will merge or create a new document automatically by Page-Writer by using numerous algorithms against the existing documents and notes to make a decision. **(this feature currently will have varying results if the database is too small, so for the best results only use this feature when there is a reasonable population of documents and notes)**

- Interactive document view page with the ability to update, delete and move individual snippets.

### Keybinds

Keybinds are a central part of Page Writers usage and allows for better efficiency when navigating the application.

**Currently keybinds are hardcoded and cannot be changed as of writing, this is planned in the future alongside a config file and a potential settings menu**

Submitting a snippet on the input menu : Ctrl + Enter

Toggle picker for current document : Ctrl + E

Toggle picker for marking a document : Ctrl + R

Toggle picker for deleting a document : ?

Switch between input and view menus : Ctrl + T

Delete selected snippet : Del?

Move selected snippet : ?

Update selected snippet : Ctrl + Enter

Settings? : Escape

### TODO

- Config file for allowing the change of core keybinds and other potential settings, there may be a plan for an in-app settings menu to allow better accessibility.

- Markdown imported and exporter, both file and directory based.

- Potentially multiple databases.

- Windows and Android support (no plans for Ios or MAC as of writing).

- Spellchecker?

- Label for what the fuzzy finder is doing.

- Toast notification system for more transparency of actions and interactivity.

- Other QoL additions that I am not thinking of.
