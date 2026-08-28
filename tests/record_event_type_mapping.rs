use evidence_registry::{EventTypeId, RecordTypeId};

#[test]
fn every_registered_event_requires_the_exact_frozen_record_type() {
    let cases: &[(u64, u16)] = &[
        (1, 1),
        (100, 3),
        (101, 4),
        (102, 5),
        (103, 6),
        (104, 7),
        (200, 12),
        (300, 30),
        (301, 31),
        (302, 32),
        (303, 32),
        (400, 40),
        (500, 50),
        (501, 50),
        (600, 62),
        (700, 71),
        (701, 72),
        (702, 72),
        (703, 72),
        (800, 80),
        (801, 81),
        (802, 82),
        (803, 83),
        (804, 84),
        (805, 85),
        (806, 86),
        (807, 87),
        (808, 88),
    ];

    for &(event_type_id, expected_record_type_id) in cases {
        let event = EventTypeId::try_from(event_type_id).unwrap();
        assert_eq!(
            event.required_record_type_id().value(),
            expected_record_type_id,
            "event_type_id={event_type_id}"
        );
    }
}

#[test]
fn record_type_registry_rejects_unassigned_and_out_of_range_ids() {
    assert!(RecordTypeId::try_from(0_u64).is_err());
    assert!(RecordTypeId::try_from(8_u64).is_err());
    assert!(RecordTypeId::try_from(91_u64).is_err());
    assert!(RecordTypeId::try_from(65_536_u64).is_err());
}
