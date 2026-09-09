use evidence_registry::{
    EnvironmentObservationRecord, EnvironmentObservationRecordInput, RecordDecodeError,
};

#[test]
fn environment_observation_round_trips_required_fields_with_absent_optionals() {
    let input = EnvironmentObservationRecordInput {
        os_name: "Windows".to_owned(),
        os_version: None,
        filesystem_reported_name: None,
        driver_details: None,
        mount_identity: None,
        volume_identity: None,
        resolved_registry_storage_identity: None,
        probe_tool_version: "evidence-registry-test".to_owned(),
        observation_limitations: vec!["no-hostile-attestation".to_owned()],
    };
    let record = EnvironmentObservationRecord::new(input.clone()).unwrap();
    assert!(
        record
            .authoritative_cbor()
            .windows(4)
            .any(|window| window == [0x18, 0x3d, 0x01, 0xa5]),
        "the type-61 body map must count all five required fields"
    );
    assert_eq!(
        EnvironmentObservationRecord::decode_authoritative(&record.authoritative_cbor()).unwrap(),
        record
    );
    assert_eq!(record.input(), &input);
}

#[test]
fn environment_observation_rejects_an_unsorted_limitations_set() {
    assert_eq!(
        EnvironmentObservationRecord::new(EnvironmentObservationRecordInput {
            os_name: "Windows".to_owned(),
            os_version: None,
            filesystem_reported_name: None,
            driver_details: None,
            mount_identity: None,
            volume_identity: None,
            resolved_registry_storage_identity: None,
            probe_tool_version: "evidence-registry-test".to_owned(),
            observation_limitations: vec!["z".to_owned(), "a".to_owned()],
        }),
        Err(RecordDecodeError)
    );
}
