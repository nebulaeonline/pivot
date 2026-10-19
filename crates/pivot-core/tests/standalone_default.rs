#![cfg(feature = "core-integration")]
use pivot_core::{
    interface::{InputRequirement, NamedInvocationRequest},
    model::{
        BackendKind, WorkingModel,
        integration::{CancellationToken, IntegrationRuntime, RuntimeConfig},
        transfer,
    },
};
use pivot_types::PivotValue;

#[test]
fn ordinary_default_shares_resources_and_allows_explicit_shutdown_and_restart() {
    let mut a = WorkingModel::new("ordinary").unwrap();
    let b = WorkingModel::new("second").unwrap();
    assert_eq!(a.backend(), BackendKind::Persistent);
    assert_eq!(b.backend(), BackendKind::Persistent);
    let x = a.add_input("x", 4.into()).unwrap();
    let y = a.add_computed("y", "x*2+1").unwrap();
    let s = a.capture_local().unwrap();
    let view = a.inspect().unwrap();
    assert!(IntegrationRuntime::shutdown_convenience().is_err());
    a.set_input(x, 10.into()).unwrap();
    assert_eq!(s.evaluate(&[y]).unwrap().values()[&y], Ok(9.into()));
    assert_eq!(
        a.capture_local().unwrap().evaluate(&[y]).unwrap().values()[&y],
        Ok(21.into())
    );
    let domain = IntegrationRuntime::with_convenience(|r| r.memory()).unwrap();
    // Several model owners, one fixed runtime: accounting is shared.
    assert!(IntegrationRuntime::with_convenience(|r| r.stats().live_owners).unwrap() >= 3);
    drop((a, b, s, view));
    IntegrationRuntime::with_convenience(|r| r.flush()).unwrap();
    IntegrationRuntime::shutdown_convenience().unwrap();
    assert_eq!(domain.stats().charged, 0);
    let restarted = WorkingModel::new("restart").unwrap();
    assert_eq!(restarted.backend(), BackendKind::Persistent);
    drop(restarted);
    IntegrationRuntime::with_convenience(|r| r.flush()).unwrap();
    IntegrationRuntime::shutdown_convenience().unwrap();
}

#[test]
fn session_models_share_admission_and_old_generations_without_activation() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    assert_eq!(
        runtime.memory().stats().limit,
        RuntimeConfig::default().allocation_budget
    );
    let mut a = WorkingModel::new_in("a", &runtime).unwrap();
    let mut b = WorkingModel::new_in("b", &runtime).unwrap();
    assert_eq!(a.backend(), BackendKind::Persistent);
    a.enable_persistent(&runtime).unwrap();
    let other_runtime = IntegrationRuntime::standalone().unwrap();
    let unchanged = a.revision();
    assert!(a.enable_persistent(&other_runtime).is_err());
    assert_eq!(a.revision(), unchanged);
    assert!(other_runtime.finish().is_ok());
    let x = a.add_input("x", 1.into()).unwrap();
    let other = b.add_input("x", 2.into()).unwrap();
    assert_ne!(x, other);
    let s = a.capture_local().unwrap();
    let original = a.revision();
    let mut branch = a.stage_expected(original).unwrap();
    branch.rename(x, "renamed").unwrap();
    branch.set_input(x, 7.into()).unwrap();
    assert_eq!(a.revision(), original);
    a = branch;
    assert!(a.stage_expected(original).is_err());
    assert_eq!(s.evaluate(&[x]).unwrap().values()[&x], Ok(1.into()));
    let memory = runtime.memory();
    runtime.flush();
    let hold = memory
        .reserve(memory.stats().limit - memory.stats().charged)
        .unwrap();
    let before = b.revision();
    assert!(b.set_input(other, 8.into()).is_err());
    assert_eq!(b.revision(), before);
    assert!(WorkingModel::new_in("refused", &runtime).is_err());
    drop(hold);
    b.set_input(other, 8.into()).unwrap();
    drop((a, b, s));
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn direct_default_ports_local_and_complete_contracts_and_transfer_are_preserved() {
    let runtime = IntegrationRuntime::standalone().unwrap();
    let mut m = runtime.create_model("direct").unwrap();
    let x = m.add_input("x", 3.into()).unwrap();
    let y = m.add_computed("y", "x+2").unwrap();
    let bad = m.add_computed("unrelated", "(").unwrap();
    m.add_input_port(
        "argument",
        x,
        InputRequirement::Optional {
            default: PivotValue::Decimal(9.into()),
        },
    )
    .unwrap();
    let output = m.add_output_port("result", y).unwrap();
    assert!(m.capture_calculation().is_err());
    let s = m.capture_local().unwrap();
    let result = s
        .invoke_named(&NamedInvocationRequest {
            inputs: vec![],
            outputs: vec!["result".into()],
        })
        .unwrap();
    assert_eq!(
        result.outputs()["result"],
        Ok(PivotValue::Decimal(11.into()))
    );
    m.replace_input(bad, 0.into()).unwrap();
    assert!(m.capture_calculation().is_ok());
    let snapshot = m.snapshot().unwrap();
    let image = transfer::export(
        &m,
        &[x, y, bad],
        &[
            m.interface()
                .inputs()
                .find(|p| p.name() == "argument")
                .unwrap()
                .id(),
            output,
        ],
    )
    .unwrap();
    let imported = runtime
        .import_portable(image, &CancellationToken::default())
        .unwrap();
    assert_eq!(imported.model.backend(), BackendKind::Persistent);
    assert!(imported.model.snapshot().is_ok());
    let mut malformed = transfer::export(
        &m,
        &[x, y, bad],
        &[
            m.interface()
                .inputs()
                .find(|p| p.name() == "argument")
                .unwrap()
                .id(),
            output,
        ],
    )
    .unwrap();
    malformed.nodes.push(malformed.nodes[0].clone());
    assert!(
        runtime
            .import_portable(malformed, &CancellationToken::default())
            .is_err()
    );
    drop((m, s, result, snapshot, imported));
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}

#[test]
fn explicit_reference_remains_available_and_runtime_admission_is_recoverable() {
    assert_eq!(
        WorkingModel::new_reference("fallback").unwrap().backend(),
        BackendKind::Reference
    );
    assert!(
        IntegrationRuntime::configured(RuntimeConfig {
            allocation_budget: 1024,
            queue_capacity: 64
        })
        .is_err()
    );
    let runtime = IntegrationRuntime::standalone().unwrap();
    let model = runtime.create_model("healthy").unwrap();
    drop(model);
    let memory = runtime.memory();
    assert!(runtime.finish().is_ok());
    assert_eq!(memory.stats().charged, 0);
}
