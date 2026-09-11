//! this_file: crates/uubed-tm/tests/memory.rs
use uubed_tm::{Pair, Store};

fn pair(source: &str, target: &str, language: &str) -> Pair {
    Pair {
        source: source.into(),
        target: target.into(),
        language: language.into(),
        origin: "sample.tmx".into(),
        unit: "1".into(),
    }
}

#[test]
fn exact_is_verbatim_and_keeps_conflicts_and_language() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("memory.sqlite");
    let mut db = Store::create(&path, "{\"space\":\"test\"}", 2).unwrap();
    db.add_pairs(&[
        pair("Save", "Zapisz", "pl"),
        pair("Save", "Zachowaj", "pl"),
        pair("Save", "Speichern", "de"),
    ])
    .unwrap();
    assert_eq!(
        db.pending(0, 20).unwrap().len(),
        1,
        "English is indexed once"
    );
    assert_eq!(
        db.exact("Save", "pl").unwrap().len(),
        2,
        "conflicting exact targets survive"
    );
    assert!(db.exact("save", "pl").unwrap().is_empty());
    assert!(db.exact("Save ", "pl").unwrap().is_empty());
    drop(db);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(reopened.exact("Save", "de").unwrap()[0].target, "Speichern");
}

#[test]
fn semantic_search_uses_signed_cosine_and_filters_targets() {
    let directory = tempfile::tempdir().unwrap();
    let mut db = Store::create(&directory.path().join("memory.sqlite"), "{}", 2).unwrap();
    db.add_pairs(&[
        pair("font", "czcionka", "pl"),
        pair("cat", "kot", "pl"),
        pair("dog", "Hund", "de"),
    ])
    .unwrap();
    db.set_vectors(&[
        (1, vec![127, 0], 0.01),
        (2, vec![129, 0], 0.01),
        (3, vec![127, 0], 0.01),
    ])
    .unwrap();
    let hits = db.search(&[1.0, 0.0], "pl", 5, -1.0).unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].source, "font");
    assert!((hits[0].score - 1.0).abs() < 1e-6);
    assert!((hits[1].score + 1.0).abs() < 1e-6);
    assert_eq!(db.search(&[1.0, 0.0], "pl", 1, 0.5).unwrap().len(), 1);
    assert!(db.search(&[1.0], "pl", 5, 0.0).is_err());
    assert!(db.search(&[f32::NAN, 0.0], "pl", 5, 0.0).is_err());
    assert!(db.search(&[0.0, 0.0], "pl", 5, 0.0).is_err());
    assert!(db.set_vectors(&[(1, vec![1], 1.0)]).is_err());
}

#[test]
fn invalid_batch_rolls_back_and_missing_database_is_not_created() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.sqlite");
    assert!(Store::open(&path).is_err());
    assert!(!path.exists());
    let mut db = Store::create(&path, "{}", 2).unwrap();
    assert!(
        db.add_pairs(&[pair("good", "dobry", "pl"), pair("", "bad", "pl")])
            .is_err()
    );
    assert!(db.exact("good", "pl").unwrap().is_empty());
    assert!(
        Store::create(&path, "{}", 2).is_err(),
        "must not overwrite an index"
    );
}
