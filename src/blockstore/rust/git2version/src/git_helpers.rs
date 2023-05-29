use git2::{Oid, Repository};
use std::collections::hash_map::{Entry, HashMap};

pub fn all_tags(repo: &Repository) -> Result<HashMap<Oid, Vec<String>>, git2::Error> {
    let mut result: HashMap<Oid, Vec<String>> = HashMap::new();
    // Because `Repository::tag_foreach` doesn't support the callback to return an error, we
    // keep a variable remembering whether an error happened and set it from the callback.
    let mut error = None;
    repo.tag_foreach(|commit_id, name| {
        let name = std::str::from_utf8(name)
            .map_err(|err| git2::Error::from_str(&format!("Tag name is not valid UTF-8: {}", err)));
        let name = match name {
            Ok(name) => name,
            Err(err) => {
                assert!(
                    error.is_none(),
                    "We immediately exit after an error so this can't be set yet"
                );
                // Set error and stop iterating
                error = Some(err);
                return false;
            }
        };
        let name = name.strip_prefix("refs/tags/").ok_or_else(|| {
            git2::Error::from_str(&format!(
                "Tag name '{}' doesn't start with 'refs/tags/'",
                name
            ))
        });
        let name = match name {
            Ok(name) => name,
            Err(err) => {
                assert!(
                    error.is_none(),
                    "We immediately exit after an error so this can't be set yet"
                );
                // Set error and stop iterating
                error = Some(err);
                return false;
            }
        };
        let name = name.to_owned();
        match result.entry(commit_id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(name);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![name]);
            }
        }
        true
    })?;

    if let Some(error) = error {
        Err(error)
    } else {
        Ok(result)
    }
}

// TODO Tests
