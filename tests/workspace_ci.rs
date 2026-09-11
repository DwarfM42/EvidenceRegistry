//! CI must qualify the companion as well as the root package.
//! These static guards preserve the executable native commands; the commands
//! themselves are exercised locally and by each actual platform workflow.
#[test]
fn each_native_workflow_includes_binder_and_preserves_core_gates() {
    let workflows = [
        ("windows", include_str!("../.github/workflows/windows.yml")),
        ("linux", include_str!("../.github/workflows/linux.yml")),
        ("macos", include_str!("../.github/workflows/macos.yml")),
    ];
    for (platform, workflow) in workflows {
        let commands: Vec<_> = workflow
            .lines()
            .filter_map(|line| line.trim().strip_prefix("- run: "))
            .collect();
        for required in [
            "cargo build --workspace --release --locked",
            "cargo fmt --all --check",
            "cargo test --workspace --all-targets --locked",
            "cargo clippy --workspace --all-targets --locked -- -D warnings",
            "cargo test --release --test review_admission_runtime --locked",
            "cargo test --release --test freeze_committed_binding --locked",
            "cargo test --release --test selected_review_demo --locked",
            "cargo test -p ai-agent-evidence-binder --release --all-targets --locked",
            "cargo test --workspace --doc --locked",
            "git diff --check",
        ] {
            assert!(
                commands
                    .iter()
                    .any(|command| *command == required
                        || command.ends_with(&format!(" && {required}"))),
                "{platform} missing {required}"
            );
        }
        assert!(workflow.contains("persist-credentials: false"));
        assert!(workflow.contains("run: git diff --exit-code"));
    }
}
