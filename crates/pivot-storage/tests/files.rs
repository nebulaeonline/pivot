use pivot_storage::{
    Decimal96, Definition, Limits, PortKind, Uuid,
    experimental_file::{self as file, Error, FileDocument, FileLimits, SavePoint, SaveTarget},
};
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

struct Directory {
    root: PathBuf,
    path: PathBuf,
}
impl Directory {
    fn new() -> Self {
        let root = if cfg!(unix) {
            std::env::var_os("PIVOT_FILE_TEST_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::temp_dir().join("pivot-experimental-file-tests"))
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/file-tests")
        };
        fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let path = root.join(Uuid::new_v4().to_string());
        fs::create_dir(&path).unwrap();
        Self { root, path }
    }
    fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        assert_eq!(self.path.parent(), Some(self.root.as_path()));
        assert!(Uuid::parse_str(self.path.file_name().unwrap().to_str().unwrap()).is_ok());
        let _ = fs::remove_dir_all(&self.path);
    }
}
fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}
fn d(n: u32) -> Decimal96 {
    Decimal96::from_parts(n, 0, 0, false, 0).unwrap()
}
fn model() -> FileDocument {
    let mut f = FileDocument::new("ProfitModel", FileLimits::default()).unwrap();
    let s = f.session_mut().unwrap();
    s.add_input(s.expected(), id(1), "Revenue", d(100)).unwrap();
    s.add_input(s.expected(), id(2), "Costs", d(25)).unwrap();
    s.add_formula(s.expected(), id(3), "Net", "Revenue - Costs")
        .unwrap();
    s.add_formula(s.expected(), id(4), "Margin", "Net / Revenue")
        .unwrap();
    s.add_port(s.expected(), id(10), "revenue", id(1), PortKind::Required)
        .unwrap();
    s.add_port(
        s.expected(),
        id(11),
        "costs",
        id(2),
        PortKind::Optional(d(25)),
    )
    .unwrap();
    s.add_port(s.expected(), id(12), "net", id(3), PortKind::Output)
        .unwrap();
    s.add_port(s.expected(), id(13), "margin", id(4), PortKind::Output)
        .unwrap();
    f
}
fn revenue(f: &mut FileDocument, n: u32) {
    let s = f.session_mut().unwrap();
    s.set_input(s.expected(), id(1), d(n)).unwrap();
}
fn net(f: &FileDocument) -> String {
    let s = f.session().unwrap();
    s.evaluate(s.expected(), &[id(3)]).unwrap().values[0]
        .1
        .as_ref()
        .unwrap()
        .runtime()
        .unwrap()
        .to_string()
}
fn resign(bytes: &mut [u8]) {
    let mut hash = Sha256::new();
    hash.update(&bytes[..36]);
    hash.update(&bytes[file::HEADER_BYTES..]);
    bytes[36..68].copy_from_slice(&hash.finalize());
}
fn child(phase: &str, path: &Path, argument: &str, copy: Option<&Path>) -> std::process::Output {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "file_process_worker", "--nocapture"])
        .env("PIVOT_FILE_PHASE", phase)
        .env("PIVOT_FILE_PATH", path)
        .env("PIVOT_FILE_ARG", argument);
    if let Some(copy) = copy {
        command.env("PIVOT_FILE_COPY", copy);
    }
    command.output().unwrap()
}
fn child_ok(phase: &str, path: &Path, arg: &str, copy: Option<&Path>) {
    let output = child(phase, path, arg, copy);
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    println!("independent process {phase}: PASS");
}

// A real executable integration harness: each child has no parent runtime state.
#[test]
fn file_process_worker() {
    let Ok(phase) = std::env::var("PIVOT_FILE_PHASE") else {
        return;
    };
    let path = PathBuf::from(std::env::var_os("PIVOT_FILE_PATH").unwrap());
    let arg = std::env::var("PIVOT_FILE_ARG").unwrap();
    if phase == "create" {
        let mut f = model();
        f.save_as(&path, SaveTarget::New).unwrap();
        f.close(false).unwrap();
        return;
    }
    let mut f = FileDocument::open(&path, FileLimits::default()).unwrap();
    assert!(!f.is_dirty());
    match phase.as_str() {
        "check" => {
            assert_eq!(net(&f), arg);
            let s = f.session().unwrap();
            let result = s
                .invoke(
                    s.expected(),
                    &[("revenue".into(), d(200))],
                    &["net".into(), "margin".into()],
                )
                .unwrap();
            assert_eq!(
                result.values[0]
                    .1
                    .as_ref()
                    .unwrap()
                    .runtime()
                    .unwrap()
                    .to_string(),
                "175"
            );
            assert_eq!(
                result.values[1]
                    .1
                    .as_ref()
                    .unwrap()
                    .runtime()
                    .unwrap()
                    .to_string(),
                "0.875"
            );
            assert!(s.invoke(s.expected(), &[], &["net".into()]).is_err());
        }
        "edit" => {
            revenue(&mut f, arg.parse().unwrap());
            f.save().unwrap();
        }
        "copy" => {
            f.save_as(
                PathBuf::from(std::env::var_os("PIVOT_FILE_COPY").unwrap()),
                SaveTarget::New,
            )
            .unwrap();
        }
        "crash-before" | "crash-after" => {
            revenue(&mut f, 200);
            let point = if phase == "crash-before" {
                SavePoint::BeforeInstall
            } else {
                SavePoint::AfterInstall
            };
            f.prepare_save().unwrap().commit_with(|p| {
                if p == point {
                    std::process::exit(70);
                }
                Ok(())
            });
            panic!("crash hook did not execute");
        }
        _ => panic!("unknown worker phase"),
    }
    f.close(false).unwrap();
}

#[test]
fn standalone_file_lifecycle_across_independent_processes() {
    let dir = Directory::new();
    let path = dir.file("profit.experimental");
    let copy = dir.file("copy.experimental");
    child_ok("create", &path, "", None);
    child_ok("check", &path, "75", None);
    child_ok("edit", &path, "200", None);
    child_ok("check", &path, "175", None);
    child_ok("copy", &path, "", Some(&copy));
    child_ok("check", &copy, "175", None);
    let a = FileDocument::open(&path, FileLimits::default()).unwrap();
    let b = FileDocument::open(&copy, FileLimits::default()).unwrap();
    assert_eq!(
        a.session().unwrap().document_id(),
        b.session().unwrap().document_id()
    );
    assert_eq!(
        a.session().unwrap().graph().model,
        b.session().unwrap().graph().model
    );
    assert_ne!(
        a.session().unwrap().expected().session,
        b.session().unwrap().expected().session
    );
    child_ok("edit", &copy, "300", None);
    child_ok("check", &copy, "275", None);
    child_ok("check", &path, "175", None);
}

#[test]
fn process_exit_before_and_after_install_preserves_complete_versions() {
    let dir = Directory::new();
    let path = dir.file("crash.experimental");
    child_ok("create", &path, "", None);
    assert_eq!(
        child("crash-before", &path, "", None).status.code(),
        Some(70)
    );
    child_ok("check", &path, "75", None);
    assert_eq!(
        child("crash-after", &path, "", None).status.code(),
        Some(70)
    );
    child_ok("check", &path, "175", None);
    println!("process termination at pre/post installation: PASS (not a power-loss test)");
}

#[test]
fn envelope_roundtrip_and_hostile_containers_fail_closed() {
    let mut f = model();
    let expected = f.session().unwrap().expected();
    let capture = f.session_mut().unwrap().export(expected).unwrap();
    let version = Uuid::new_v4();
    let bytes = file::encode(capture.document(), version, FileLimits::default()).unwrap();
    let (v, doc) = file::decode(&bytes, FileLimits::default()).unwrap();
    assert_eq!(v, version);
    assert_eq!(&doc, capture.document());
    for end in 0..bytes.len() {
        assert!(file::decode(&bytes[..end], FileLimits::default()).is_err());
    }
    let mut bad = bytes.clone();
    bad[8..12].copy_from_slice(&99u32.to_be_bytes());
    assert!(matches!(
        file::decode(&bad, FileLimits::default()),
        Err(Error::UnsupportedVersion)
    ));
    let mut bad = bytes.clone();
    bad[36] ^= 1;
    assert!(matches!(
        file::decode(&bad, FileLimits::default()),
        Err(Error::Integrity)
    ));
    let mut bad = bytes.clone();
    bad[12..20].copy_from_slice(&u64::MAX.to_be_bytes());
    assert!(matches!(
        file::decode(&bad, FileLimits::default()),
        Err(Error::Limit)
    ));
    let mut bad = bytes.clone();
    bad[file::HEADER_BYTES] = 0x83;
    resign(&mut bad);
    assert!(file::decode(&bad, FileLimits::default()).is_err());
    let mut bad = bytes.clone();
    let offset = bad
        .windows(16)
        .rposition(|w| w == doc.graph.model.as_bytes())
        .unwrap();
    bad[offset..offset + 16].copy_from_slice(Uuid::new_v4().as_bytes());
    resign(&mut bad);
    assert!(file::decode(&bad, FileLimits::default()).is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(file::decode(&trailing, FileLimits::default()).is_err());
    assert!(
        file::decode(
            &bytes,
            FileLimits {
                payload_bytes: 1,
                ..FileLimits::default()
            }
        )
        .is_err()
    );
    let dir = Directory::new();
    let path = dir.file("bad.experimental");
    fs::write(&path, bad).unwrap();
    assert!(FileDocument::open(&path, FileLimits::default()).is_err());
}

#[test]
fn bounded_arbitrary_envelopes_never_panic() {
    let mut state = 123u64;
    for n in 0..512 {
        let bytes: Vec<_> = (0..n)
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                (state >> 32) as u8
            })
            .collect();
        assert!(file::decode(&bytes, FileLimits::default()).is_err());
    }
}

#[test]
fn stable_bindings_drafts_and_retired_allocations_survive_files() {
    let dir = Directory::new();
    let path = dir.file("draft.experimental");
    let mut f = model();
    let s = f.session_mut().unwrap();
    s.rename(s.expected(), id(1), "Sales").unwrap();
    s.add_input(s.expected(), id(5), "Revenue", d(999)).unwrap();
    let definition = s.graph().nodes[2].definition.clone();
    f.save_as(&path, SaveTarget::New).unwrap();
    let mut f = FileDocument::open(&path, FileLimits::default()).unwrap();
    assert_eq!(net(&f), "75");
    assert_eq!(f.session().unwrap().graph().nodes[2].definition, definition);
    let s = f.session_mut().unwrap();
    s.remove_port(s.expected(), id(10)).unwrap();
    s.remove(s.expected(), id(1)).unwrap();
    s.set_formula(s.expected(), id(3), "Revenue +").unwrap();
    f.save().unwrap();
    let mut f = FileDocument::open(&path, FileLimits::default()).unwrap();
    assert!(f.session().unwrap().graph().node_ids.contains(&id(1)));
    assert!(f.session().unwrap().graph().port_ids.contains(&id(10)));
    assert!(
        matches!(&f.session().unwrap().graph().nodes[1].definition,Definition::ParseFailed {source,..} if source=="Revenue +")
    );
    let s = f.session_mut().unwrap();
    assert!(s.add_input(s.expected(), id(1), "Reuse", d(1)).is_err());
    s.set_formula(s.expected(), id(3), "Revenue - Costs")
        .unwrap();
    s.set_formula(s.expected(), id(4), "Net / Revenue").unwrap();
    assert_eq!(net(&f), "974");
    f.save().unwrap();
}

#[test]
fn exact_decimal_representations_roundtrip_on_disk() {
    let dir = Directory::new();
    let path = dir.file("exact.experimental");
    let mut f = FileDocument::new("Exact", FileLimits::default()).unwrap();
    let s = f.session_mut().unwrap();
    let mut representations = vec![];
    for scale in 0..=28 {
        for negative in [false, true] {
            let v = Decimal96::from_parts(0, 0, 0, negative, scale).unwrap();
            let n = representations.len() as u128 + 1;
            s.add_input(s.expected(), id(n), &format!("N{n}"), v)
                .unwrap();
            representations.push(v);
        }
    }
    let maximum = Decimal96::from_parts(u32::MAX, u32::MAX, u32::MAX, true, 28).unwrap();
    s.add_input(s.expected(), id(100), "Maximum", maximum)
        .unwrap();
    s.add_port(
        s.expected(),
        id(101),
        "Default",
        id(1),
        PortKind::Optional(representations[57]),
    )
    .unwrap();
    s.add_formula(s.expected(), id(102), "Literal", "N1 + 1.2300")
        .unwrap();
    let before = s.graph().clone();
    f.save_as(&path, SaveTarget::New).unwrap();
    let mut f = FileDocument::open(&path, FileLimits::default()).unwrap();
    assert_eq!(f.session().unwrap().graph(), &before);
    let s = f.session_mut().unwrap();
    s.rename(s.expected(), id(100), "Renamed").unwrap();
    f.save().unwrap();
    let f = FileDocument::open(&path, FileLimits::default()).unwrap();
    for (n, v) in representations.iter().enumerate() {
        assert_eq!(
            f.session().unwrap().graph().nodes[n].definition,
            Definition::Input(*v)
        );
    }
    assert_eq!(
        f.session().unwrap().graph().ports[0].kind,
        PortKind::Optional(representations[57])
    );
}

#[test]
fn preinstall_faults_leave_original_bytes_and_association() {
    let dir = Directory::new();
    let path = dir.file("original.experimental");
    let mut f = model();
    f.save_as(&path, SaveTarget::New).unwrap();
    let original = fs::read(&path).unwrap();
    let association = f.association().unwrap().fingerprint().clone();
    revenue(&mut f, 200);
    for point in [
        SavePoint::BeforeEncode,
        SavePoint::BeforeWrite,
        SavePoint::PartialWrite,
        SavePoint::BeforeInstall,
    ] {
        let outcome = f.prepare_save().unwrap().commit_with(|p| {
            if p == point {
                Err(io::Error::other("injected fault"))
            } else {
                Ok(())
            }
        });
        assert!(f.finish_save(&outcome).is_err());
        drop(outcome);
        assert!(f.is_dirty());
        assert_eq!(fs::read(&path).unwrap(), original);
        assert_eq!(f.association().unwrap().fingerprint(), &association);
        assert!(
            !fs::read_dir(&dir.path).unwrap().any(|p| p
                .unwrap()
                .path()
                .extension()
                .is_some_and(|e| e == "tmp"))
        );
    }
}

#[test]
fn encoder_budget_failure_never_touches_existing_file() {
    let dir = Directory::new();
    let path = dir.file("bounded.experimental");
    let mut f = model();
    f.save_as(&path, SaveTarget::New).unwrap();
    let bytes = fs::read(&path).unwrap();
    let mut f = FileDocument::open(
        &path,
        FileLimits {
            payload_bytes: bytes.len() - file::HEADER_BYTES + 1,
            graph: Limits::default(),
        },
    )
    .unwrap();
    let s = f.session_mut().unwrap();
    s.add_formula(s.expected(), id(50), "Extra", "Revenue + Costs + 1")
        .unwrap();
    assert!(f.save().is_err());
    assert!(f.is_dirty());
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn postinstall_failures_remain_uncertain_until_verified_reconciliation() {
    for point in [SavePoint::AfterInstall, SavePoint::BeforeVerify] {
        let dir = Directory::new();
        let path = dir.file("uncertain.experimental");
        let mut f = model();
        f.save_as(&path, SaveTarget::New).unwrap();
        let baseline = f.association().unwrap().fingerprint().clone();
        revenue(&mut f, 200);
        let plan = f.prepare_save().unwrap();
        revenue(&mut f, 300);
        let outcome = plan.commit_with(|p| {
            if p == point {
                Err(io::Error::other("after replacement"))
            } else {
                Ok(())
            }
        });
        assert!(matches!(f.finish_save(&outcome), Err(Error::Uncertain(_))));
        drop(outcome);
        assert!(f.is_dirty());
        assert_eq!(f.association().unwrap().fingerprint(), &baseline);
        assert!(matches!(f.prepare_save(), Err(Error::Pending)));
        assert_eq!(
            net(&FileDocument::open(&path, FileLimits::default()).unwrap()),
            "175"
        );
        f.reconcile().unwrap();
        assert!(f.is_dirty());
        assert_eq!(net(&f), "275");
        f.save().unwrap();
        assert!(!f.is_dirty());
    }
}

#[test]
fn verified_s_save_leaves_newer_t_dirty() {
    let dir = Directory::new();
    let path = dir.file("snapshot.experimental");
    let mut f = model();
    let plan = f.prepare_save_as(&path, SaveTarget::New).unwrap();
    revenue(&mut f, 200);
    let outcome = plan.commit();
    f.finish_save(&outcome).unwrap();
    drop(outcome);
    assert!(f.is_dirty());
    assert_eq!(net(&f), "175");
    assert_eq!(
        net(&FileDocument::open(&path, FileLimits::default()).unwrap()),
        "75"
    );
    assert!(matches!(f.close(false), Err(Error::Dirty)));
    f.close(true).unwrap();
    assert!(matches!(f.session(), Err(Error::Closed)));
}

#[test]
fn external_modification_is_detected_even_with_same_length_and_timestamp() {
    let dir = Directory::new();
    let path = dir.file("conflict.experimental");
    let mut a = model();
    a.save_as(&path, SaveTarget::New).unwrap();
    let stamp = fs::metadata(&path).unwrap().modified().unwrap();
    let original_len = fs::metadata(&path).unwrap().len();
    let mut other = FileDocument::open(&path, FileLimits::default()).unwrap();
    revenue(&mut other, 200);
    other.save().unwrap();
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(stamp))
        .unwrap();
    assert_eq!(fs::metadata(&path).unwrap().len(), original_len);
    let externally_saved = fs::read(&path).unwrap();
    revenue(&mut a, 300);
    assert!(matches!(a.save(), Err(Error::Conflict)));
    assert!(a.is_dirty());
    assert_eq!(fs::read(&path).unwrap(), externally_saved);
}

#[test]
fn save_as_requires_observed_consent_and_failure_preserves_association() {
    let dir = Directory::new();
    let original = dir.file("original.experimental");
    let copy = dir.file("copy.experimental");
    let mut f = model();
    f.save_as(&original, SaveTarget::New).unwrap();
    let associated = f.association().unwrap().path().to_owned();
    fs::write(&copy, b"user file").unwrap();
    assert!(matches!(
        f.save_as(&copy, SaveTarget::New),
        Err(Error::Conflict)
    ));
    assert_eq!(fs::read(&copy).unwrap(), b"user file");
    assert_eq!(f.association().unwrap().path(), associated);
    assert!(
        f.save_as(dir.file("missing/child"), SaveTarget::New)
            .is_err()
    );
    assert_eq!(f.association().unwrap().path(), associated);
    let observed = file::observe(&copy, FileLimits::default()).unwrap();
    f.save_as(&copy, SaveTarget::ReplaceObserved(observed))
        .unwrap();
    assert_eq!(
        net(&FileDocument::open(&copy, FileLimits::default()).unwrap()),
        "75"
    );
}

#[test]
fn new_target_race_is_not_clobbered() {
    let dir = Directory::new();
    let path = dir.file("race.experimental");
    let mut f = model();
    let outcome = f
        .prepare_save_as(&path, SaveTarget::New)
        .unwrap()
        .commit_with(|p| {
            if p == SavePoint::BeforeInstall {
                fs::write(&path, b"external new file")?;
            }
            Ok(())
        });
    assert!(matches!(f.finish_save(&outcome), Err(Error::Conflict)));
    drop(outcome);
    assert!(f.association().is_none());
    assert_eq!(fs::read(&path).unwrap(), b"external new file");
}

#[test]
fn cooperative_reservation_is_held_through_acknowledgment() {
    let dir = Directory::new();
    let mut a = model();
    let mut b = model();
    let first = a
        .prepare_save_as(dir.file("a.experimental"), SaveTarget::New)
        .unwrap()
        .commit();
    let second = b
        .prepare_save_as(dir.file("b.experimental"), SaveTarget::New)
        .unwrap()
        .commit();
    assert!(matches!(b.finish_save(&second), Err(Error::Busy)));
    drop(second);
    a.finish_save(&first).unwrap();
    drop(first);
    b.save_as(dir.file("b.experimental"), SaveTarget::New)
        .unwrap();
}

#[cfg(unix)]
#[test]
fn unix_private_modes_and_alias_rejection() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = Directory::new();
    let path = dir.file("mode.experimental");
    let mut f = model();
    f.save_as(&path, SaveTarget::New).unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
    let mut f = FileDocument::open(&path, FileLimits::default()).unwrap();
    revenue(&mut f, 200);
    f.save().unwrap();
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o640
    );
    let link = dir.file("symlink");
    symlink(&path, &link).unwrap();
    assert!(FileDocument::open(&link, FileLimits::default()).is_err());
    let link = dir.file("hardlink");
    fs::hard_link(&path, &link).unwrap();
    assert!(FileDocument::open(&link, FileLimits::default()).is_err());
}

#[test]
fn uncertain_save_as_keeps_old_association_until_reconciled() {
    let dir = Directory::new();
    let old = dir.file("old.experimental");
    let new = dir.file("new.experimental");
    let mut f = model();
    f.save_as(&old, SaveTarget::New).unwrap();
    let original = fs::read(&old).unwrap();
    let old_path = f.association().unwrap().path().to_owned();
    revenue(&mut f, 200);
    let outcome = f
        .prepare_save_as(&new, SaveTarget::New)
        .unwrap()
        .commit_with(|p| {
            if p == SavePoint::AfterInstall {
                Err(io::Error::other("lost completion"))
            } else {
                Ok(())
            }
        });
    assert!(matches!(f.finish_save(&outcome), Err(Error::Uncertain(_))));
    drop(outcome);
    assert_eq!(f.association().unwrap().path(), old_path);
    assert_eq!(fs::read(&old).unwrap(), original);
    assert!(f.is_dirty());
    f.reconcile().unwrap();
    assert!(!f.is_dirty());
    assert_eq!(f.association().unwrap().path(), new.canonicalize().unwrap());
    assert_eq!(
        net(&FileDocument::open(&new, FileLimits::default()).unwrap()),
        "175"
    );
}

#[test]
fn unresolved_save_never_acknowledges_different_installed_content() {
    let dir = Directory::new();
    let path = dir.file("changed.experimental");
    let mut f = model();
    f.save_as(&path, SaveTarget::New).unwrap();
    revenue(&mut f, 200);
    let outcome = f.prepare_save().unwrap().commit_with(|p| {
        if p == SavePoint::BeforeVerify {
            Err(io::Error::other("verification fault"))
        } else {
            Ok(())
        }
    });
    assert!(matches!(f.finish_save(&outcome), Err(Error::Uncertain(_))));
    drop(outcome);
    let mut external = FileDocument::open(&path, FileLimits::default()).unwrap();
    revenue(&mut external, 300);
    external.save().unwrap();
    assert!(matches!(f.reconcile(), Err(Error::Conflict)));
    assert!(f.is_dirty());
    assert!(matches!(f.prepare_save(), Err(Error::Pending)));
    f.close(true).unwrap();
}

#[test]
fn prepared_capture_can_be_canceled_and_reserved_paths_are_rejected() {
    let dir = Directory::new();
    let path = dir.file("new.experimental");
    let mut f = model();
    assert!(matches!(f.save(), Err(Error::NeedsPath)));
    let plan = f.prepare_save_as(&path, SaveTarget::New).unwrap();
    assert!(matches!(f.close(true), Err(Error::Pending)));
    f.cancel_prepared(plan).unwrap();
    assert!(!path.exists());
    assert!(
        f.prepare_save_as(dir.file(".pivot-experimental-write.lock"), SaveTarget::New)
            .is_err()
    );
    f.save_as(&path, SaveTarget::New).unwrap();
    f.close(false).unwrap();
    assert!(matches!(f.session(), Err(Error::Closed)));
}
