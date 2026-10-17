use anyhow::Result;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;

pub mod sqlite_interface;

pub type CorpusSnippets = HashMap<String, Vec<String>>;
pub type Corpus = HashMap<String, String>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DocumentSnippet {
    document_id: i32,
    document_name: String,
    snippets: Vec<SnippetEntry>,
}

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug, Clone)]
pub struct SnippetEntry {
    snippet_id: i32,
    snippet: String,
    // a stringified Instant
    last_modified: String,
}

pub async fn submit_snippet(
    snippet: &str,
    title: Option<&str>,
    db: &SqlitePool,
) -> Result<(), anyhow::Error> {
    if snippet.is_empty() {
        println!("Snippet is empty");

        return Ok(());
    };

    if let Some(title) = title {
        sqlite_interface::add_document(db, None, title.trim(), vec![snippet.to_owned()]).await?;

        Ok(())
    } else {
        let marked_document = sqlite_interface::fetch_marked_document(db).await?;

        if let Some(marked_document) = marked_document {
            println!("Marked doc found");
            sqlite_interface::add_document(db, None, &marked_document, vec![snippet.to_owned()])
                .await?;

            return Ok(());
        }

        println!("No marked doc found");
        let first_line = snippet.lines().collect::<Vec<&str>>()[0];
        sqlite_interface::add_document(db, None, first_line.trim(), vec![snippet.to_owned()])
            .await?;
        sqlite_interface::set_marked_document(db, first_line.trim()).await?;

        Ok(())
    }
}

pub async fn update_snippet(
    db: &SqlitePool,
    snippet_id: i32,
    snippet: &str,
) -> Result<(), anyhow::Error> {
    if snippet.is_empty() {
        println!("Snippet is empty");
        sqlite_interface::delete_snippet(db, snippet_id).await?;
        return Ok(());
    };

    sqlite_interface::update_snippet(db, snippet_id, snippet).await?;

    Ok(())
}

// needs to grab both files 'remote_data' and 'local_data' and synchronise the local with the remote
// of course, needs to ensure a copy of the original data is saved as a backup (needs mechanism /
// dialogue to trigger)
//
// first thing to look at is the database schema needs to be the same
// the data being saved must not be a duplicate (probably simple similarity check)
//
// first idea (however slow) is to sequentually go through each entry of the database by key order
// for each document, if there are matching keys then check if the content is the same, if not the
// hold onto the mismatched snippet and add them on at the end or override that keyed snippet with
// the remote one. As you cant add snippets with keys that are before existing ones (like creating a
// record with key 1 if 2 exists) it should be easy to identify what has been deleted and what has
// been updated if the remote has or doesnt have a record with a matching id to the one in the local
// save. This could be done in threads for each document, although the async nature of the db makes this a very bad idea
//
//
// the sync must prioritise adding snippets over deleting, if the remote is empty then the local
// data should not also be erased and made empty
//
// NOTE: remember this data sync is not true sync with active communication between devices, this is
// only to replicate sync with the use of a file we want to match and a copy of the data we want
// modified. In essence, we only care about what the local data will look like, no modifying
// operations needed on remote_data and only modify local_data

// entry point for other methods in place
// need to decide if the syncing is done on the database itself or through modifying a hashmap that
// will be used to update the database.
//
// I feel the former is better
// syncs data through modifying the remote database or through modifying the data obtained from
// them, which is used to override original
// might be more efficient to hold on set of data for remote data and modify local database, but it
// will be harder to sort out.
pub async fn sync_database(local_db: &SqlitePool, remote_db: &SqlitePool) -> Result<()> {
    let local_data: HashMap<(i32, String), DocumentSnippet> =
        sqlite_interface::load_all_documents(&local_db)
            .await?
            .into_iter()
            .map(|d| ((d.document_id, d.document_name.clone()), d))
            .collect();
    let remote_data: HashMap<(i32, String), DocumentSnippet> =
        sqlite_interface::load_all_documents(&remote_db)
            .await?
            .into_iter()
            .map(|d| ((d.document_id, d.document_name.clone()), d))
            .collect();

    if remote_data.is_empty() {
        return Ok(());
    }

    // copy all remote data into the empty local database
    if local_data.is_empty() {
        for (_, document) in remote_data {
            sqlite_interface::add_document(
                local_db,
                Some(document.document_id),
                &document.document_name,
                document
                    .snippets
                    .iter()
                    .map(|s| s.snippet.clone())
                    .collect::<Vec<String>>(),
            )
            .await?;
        }

        return Ok(());
    }

    // need a way to filter the data to contain those with matching documents
    let mutual_entries: Vec<(i32, String)> = local_data
        .iter()
        .filter(|(k, _)| remote_data.contains_key(k))
        .map(|(k, _)| k.to_owned())
        .collect();

    // probably redundant
    let different_local_entries: Vec<(i32, String)> = local_data
        .iter()
        .filter(|(k, _)| !remote_data.contains_key(k))
        .map(|(k, _)| k.to_owned())
        .collect();

    // The data to be synced into local
    let different_remote_entries: Vec<(i32, String)> = remote_data
        .iter()
        .filter(|(k, _)| !local_data.contains_key(k))
        .map(|(k, _)| k.to_owned())
        .collect();

    // Order of search
    // - Exact Document name and Key combination exists
    // - Document name exists
    // - Key exists / neither exists then create new document
    //
    // overall, prioritise the document name and ignore the key even if it is different between
    // local and remote.

    // exact key and document name exists
    for entry in mutual_entries {
        sync_document(
            local_data
                .get(&entry)
                .expect("Unexpected key, entry not found")
                .to_owned(),
            remote_data
                .get(&entry)
                .expect("Unexpected key, entry not found")
                .to_owned(),
            local_db,
        )
        .await?;
    }

    // if a remote document exists but local one doesn't, simply add the document in with exact
    // snippets
    for entry in different_remote_entries {
        if different_local_entries.iter().any(|(_, d)| &entry.1 == d) {
            let local_entry: Vec<(i32, String)> = different_local_entries
                .iter()
                .filter(|(_, d)| &entry.1 == d)
                .map(|e| e.to_owned())
                .collect();

            sync_document(
                local_data
                    .get(&local_entry[0])
                    .expect("Unexpected key, entry not found")
                    .to_owned(),
                remote_data
                    .get(&entry)
                    .expect("Unexpected key, entry not found")
                    .to_owned(),
                local_db,
            )
            .await?;
        }
    }

    Ok(())
}

async fn sync_document(
    local_document: DocumentSnippet,
    remote_document: DocumentSnippet,
    local_db: &SqlitePool,
) -> Result<()> {
    // check if IDs exist, then check if the contents are the same, otherwise update them
    // most likely if an id exists but the contents are the same, then they must have been updated
    // by the user
    //
    // if an ID doesn't exist for a remote snippet, then the local version of the snippet must have
    // been deleted. If remote snippet local snippet ID doesn't exist on remote, then prioritise
    // maintaining data

    if local_document.snippets.is_empty() && remote_document.snippets.is_empty() {
        return Ok(());
    } else if local_document.snippets.is_empty() {
        for snippet in remote_document.snippets.clone() {
            sqlite_interface::add_snippet(
                local_db,
                &snippet.snippet,
                &remote_document.document_name,
            )
            .await?;
        }
    }

    let remote_snippets: HashMap<i32, SnippetEntry> = remote_document
        .snippets
        .iter()
        .filter(|remote_s| {
            local_document
                .snippets
                .iter()
                .any(|local_s| local_s.snippet_id == remote_s.snippet_id)
        })
        .map(|remote_s| (remote_s.snippet_id, remote_s.to_owned()))
        .collect();

    // updates snippet if they are out of date (according to if last_modified is before the date on
    // the remote version, or if the snippet is missing a last_modified value)
    let mutual_snippets: Vec<(SnippetEntry, SnippetEntry)> = local_document
        .snippets
        .iter()
        .filter(|local_s| remote_snippets.contains_key(&local_s.snippet_id))
        .map(|local_s| {
            (
                local_s.to_owned(),
                remote_snippets.get(&local_s.snippet_id).unwrap().to_owned(),
            )
        })
        .collect();

    for snippet in mutual_snippets {
        // needs to decide what to do if there isn't any date specificed
        let local_modified_time = DateTime::parse_from_rfc3339(&snippet.0.last_modified);
        let remote_modified_time = DateTime::parse_from_rfc3339(&snippet.1.last_modified);

        if local_modified_time.is_ok() && remote_modified_time.is_ok() {
            if local_modified_time
                .unwrap()
                .lt(&remote_modified_time.unwrap())
            {
                update_snippet(&local_db, snippet.0.snippet_id, &snippet.1.snippet).await?;
            }
        } else if local_modified_time.is_err() && remote_modified_time.is_ok() {
            update_snippet(&local_db, snippet.0.snippet_id, &snippet.1.snippet).await?;
        }
    }

    Ok(())
}
