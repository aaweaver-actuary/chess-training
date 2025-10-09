#[test]
fn main_runs_without_panicking() {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_scheduler-core"))
        .status()
        .expect("binary should run");
    assert!(status.success());
}
