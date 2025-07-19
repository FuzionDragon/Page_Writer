# Page Writer

A powerful note taking application for spontanious note takers and power users.

Notes are treated as snippets of text that belongs to a collection of other related snippets, making up a document.

They are currently pieces of markdown which is stored in a Sqlite database, **however there are plans for exporting and importing actual markdown files**.

### Core Features

- The user has the ability to quickly and easily merge notes with a specific document by simply giving a name of an existing document, if it doesn't exist then a document with the corresponding name will be created.

- Notes without an entered name will merge or create a new document automatically by Page-Writer by using numerous algorithms against the existing documents and notes to make a decision. **(this feature will have varying results if the database is too small, so for the best results only use this feature when there is a reasonable population of documents and notes)**

### TODO

- Fuzzy finder menu for use in setting a marked document or which document to view.

- Markdown imported and exporter, both file and directory based.

- Snippet moving, updating and deleting.
