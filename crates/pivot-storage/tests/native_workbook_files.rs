use pivot_storage::experimental_file::{Error, SavePoint, native};
use std::{fs, io};
#[test]
fn staged_failure_conflict_and_uncertain_installation_preserve_evidence() {
    let root = std::env::var_os("PIVOT_FILE_TEST_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("pivot-native-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("book.pivot");
    let initial = native::destination(&path).unwrap();
    let location = native::install(&initial, b"generation S").unwrap();
    for point in [
        SavePoint::BeforeWrite,
        SavePoint::PartialWrite,
        SavePoint::BeforeInstall,
    ] {
        let result = native::install_with(&location, b"generation T", |at| {
            if at == point {
                Err(io::Error::other("injected"))
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"generation S");
    }
    fs::write(&path, b"external edit").unwrap();
    assert!(matches!(
        native::install(&location, b"T"),
        Err(Error::Conflict)
    ));
    assert_eq!(fs::read(&path).unwrap(), b"external edit");
    let observed = native::destination(&path).unwrap();
    assert!(matches!(
        native::install_with(&observed, b"complete T", |at| {
            if at == SavePoint::AfterInstall {
                Err(io::Error::other("lost acknowledgement"))
            } else {
                Ok(())
            }
        }),
        Err(Error::Uncertain(_))
    ));
    assert_eq!(native::read(&path).unwrap().0, b"complete T");
    let symlink_candidate = root.join("other.pivot");
    let new = native::destination(&symlink_candidate).unwrap();
    fs::write(&symlink_candidate, b"race").unwrap();
    assert!(matches!(native::install(&new, b"T"), Err(Error::Conflict)));
    assert_eq!(fs::read(symlink_candidate).unwrap(), b"race");
    assert!(
        fs::read_dir(&root).unwrap().all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp"))
    );
}

#[test]
fn native_crash_child() {
    let Some(root) = std::env::var_os("PIVOT_NATIVE_CRASH_ROOT") else {
        return;
    };
    let location = native::destination(&std::path::PathBuf::from(root).join("book.pivot")).unwrap();
    let _ = native::install_with(&location, b"never installed", |at| {
        if at == SavePoint::PartialWrite {
            std::process::exit(17)
        }
        Ok(())
    });
    panic!("crash hook not reached");
}
#[test]
fn terminated_writer_never_truncates_the_previous_generation() {
    let root = std::env::var_os("PIVOT_FILE_TEST_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("pivot-crash-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let path = root.join("book.pivot");
    native::install(&native::destination(&path).unwrap(), b"valid S").unwrap();
    let exit = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "native_crash_child", "--nocapture"])
        .env("PIVOT_NATIVE_CRASH_ROOT", &root)
        .status()
        .unwrap();
    assert_eq!(exit.code(), Some(17));
    assert_eq!(native::read(&path).unwrap().0, b"valid S");
    // Process death may leave a private orphan staging file; it is never opened as the document.
}
