//! Store-owned portable parameters and generic event-400 registration.
//! Schema v0.3 §§45.1,47–50; closure Core §§3,5. Registration does not evaluate
//! the Policy, and portable definitions do not attest to Method/Check execution.
use super::*;

impl AuthoritativeRegistryStore {
    /// Stages exact typed SCOPE bytes without appending any Journal event.
    /// An already retained identity is validated and reused without a new write
    /// or durability receipt; the returned RecordId is an identity only.
    pub fn stage_scope_record(
        &mut self,
        record: &ScopeRecord,
    ) -> Result<RecordId, StoreProvisioningError> {
        self.stage_portable_record(record.record_id(), record.authoritative_cbor(), None)
    }

    /// Stages a portable METHOD definition, not evidence of its execution.
    /// Existing identities are validation-only reuse, not new durability receipts.
    pub fn stage_method_record(
        &mut self,
        record: &MethodRecord,
    ) -> Result<RecordId, StoreProvisioningError> {
        self.stage_portable_record(record.record_id(), record.authoritative_cbor(), None)
    }

    /// Stages a portable CHECK definition, not a check result.
    /// Existing identities are validation-only reuse, not new durability receipts.
    pub fn stage_check_record(
        &mut self,
        record: &CheckRecord,
    ) -> Result<RecordId, StoreProvisioningError> {
        self.stage_portable_record(record.record_id(), record.authoritative_cbor(), None)
    }

    /// Resolves every member as CHECK before staging the nonempty canonical set.
    /// Existing identities are validation-only reuse, not new durability receipts.
    pub fn stage_check_set_record(
        &mut self,
        record: &CheckSetRecord,
    ) -> Result<RecordId, StoreProvisioningError> {
        self.stage_portable_record(
            record.record_id(),
            record.authoritative_cbor(),
            Some(record),
        )
    }

    // Preserve the platform's owned guard (File on Windows, lock guard on Linux)
    // through the caller's publication and readback, without borrowing the Store.
    fn provisioning_lock(&mut self) -> Result<impl Sized, StoreProvisioningError> {
        if self.open_profile
            != AuthoritativeRegistryStoreOpenProfile::SelectedTerminalAuthorityClosure
        {
            return Err(StoreProvisioningError::SelectedProfileRequired);
        }
        let root_hold = self
            .namespace_holds
            .first()
            .ok_or(StoreProvisioningError::RetainedGenerationChanged)?;
        let lock = acquire_selected_authoritative_publication_lock(&self.root, root_hold)
            .map_err(|_| StoreProvisioningError::PublicationLockUnavailable)?;
        self.reload_authoritative_namespaces().map_err(map_reload)?;
        Ok(lock)
    }

    fn stage_portable_record(
        &mut self,
        id: RecordId,
        bytes: Vec<u8>,
        checks: Option<&CheckSetRecord>,
    ) -> Result<RecordId, StoreProvisioningError> {
        let _lock = self.provisioning_lock()?;
        let head = self.retained_journal.current_head_reference();
        if let Some(checks) = checks {
            validate_check_members(self, checks)
                .map_err(|_| StoreProvisioningError::InvalidParameterReference)?;
        }
        self.provisioning_record_budget(id, &bytes, None)?;
        // Same-identity staging is validation-only reuse, not a newly issued
        // publication/durability receipt. Never reopen or rewrite those bytes.
        match self.resolve(id) {
            Some(retained) if retained == bytes => {}
            Some(_) => return Err(StoreProvisioningError::ReplayMismatch),
            None => {
                publish_record_bytes(self, id, &bytes)
                    .map_err(|_| StoreProvisioningError::RecordPublication)?;
            }
        }
        self.revalidate_retained_generation()
            .map_err(|_| StoreProvisioningError::RetainedGenerationChanged)?;
        self.reload_authoritative_namespaces().map_err(map_reload)?;
        if self.resolve(id) != Some(bytes.as_slice())
            || self.retained_journal.current_head_reference() != head
        {
            return Err(StoreProvisioningError::ReplayMismatch);
        }
        Ok(id)
    }

    /// Registers the selected minimal Freeze Policy (exact contexts [1]).
    /// Store captures the registration's current head while locked; no caller
    /// Record body, operation-start reference, or Journal slot is accepted.
    /// Event 400 is generic registration, not Policy satisfaction or a Freeze.
    pub fn register_selected_minimal_policy(
        &mut self,
        gate_scope_ref: RecordId,
    ) -> Result<JournalReference, StoreProvisioningError> {
        let _lock = self.provisioning_lock()?;
        let start = self.retained_journal.current_head_reference();
        let policy = MinimalPolicyRecord::new(MinimalPolicyRecordInput {
            gate_scope_ref,
            supported_context_ids: vec![1],
            operation_start_journal_ref: start,
        })
        .map_err(|_| StoreProvisioningError::InvalidRecord)?;
        if !selected_freeze_policy_is_supported(&policy, self) {
            return Err(StoreProvisioningError::UnsupportedPolicy);
        }
        self.publish_registered_policy(
            policy.record_id(),
            policy.authoritative_cbor(),
            vec![gate_scope_ref],
        )
    }

    /// Registers PR with exact selected contexts [2,5], before any Request or
    /// Freeze is required. Only semantic selectors/requirements are supplied.
    /// Gate/selector equality and submitted-result satisfaction are not tested
    /// here: those belong to the operation's applicable Policy evaluators.
    pub fn register_selected_review_policy(
        &mut self,
        gate_scope_ref: RecordId,
        review_requirements: Vec<ReviewAdmissionReviewRequirement>,
        required_method_statuses: Option<Vec<u64>>,
        allowed_finding_states: Option<Vec<u64>>,
        acceptable_anchor_relation_ids: Option<Vec<u64>>,
    ) -> Result<JournalReference, StoreProvisioningError> {
        let _lock = self.provisioning_lock()?;
        let start = self.retained_journal.current_head_reference();
        let policy = ReviewAdmissionPolicyRecord::new(ReviewAdmissionPolicyRecordInput {
            gate_scope_ref,
            review_requirements,
            required_method_statuses,
            allowed_finding_states,
            acceptable_anchor_relation_ids,
            supported_context_ids: vec![2, 5],
            operation_start_journal_ref: start,
        })
        .map_err(|_| StoreProvisioningError::InvalidRecord)?;
        if !selected_review_policy_is_supported(&policy, self) {
            return Err(StoreProvisioningError::UnsupportedPolicy);
        }
        let mut identities = vec![gate_scope_ref];
        for requirement in policy.review_requirements() {
            validate_review_parameters(
                self,
                requirement.review_scope_ref(),
                requirement.review_method_ref(),
                requirement.required_checks_ref(),
            )
            .map_err(|_| StoreProvisioningError::InvalidParameterReference)?;
            identities.extend([
                requirement.review_scope_ref(),
                requirement.review_method_ref(),
                requirement.required_checks_ref(),
            ]);
        }
        self.publish_registered_policy(policy.record_id(), policy.authoritative_cbor(), identities)
    }

    fn provisioning_record_budget(
        &self,
        id: RecordId,
        bytes: &[u8],
        journal: Option<&[u8]>,
    ) -> Result<(), StoreProvisioningError> {
        if bytes.len() > AUTHORITATIVE_STORE_MAX_OBJECT_BYTES
            || journal.is_some_and(|entry| entry.len() > AUTHORITATIVE_STORE_MAX_OBJECT_BYTES)
        {
            return Err(StoreProvisioningError::ResourceLimit);
        }
        let new_record = self.resolve(id).is_none();
        let new_objects = usize::from(new_record) + usize::from(journal.is_some());
        let added_bytes =
            (if new_record { bytes.len() } else { 0 }).checked_add(journal.map_or(0, <[u8]>::len));
        let total = self
            .retained_file_witnesses
            .iter()
            .try_fold(0usize, |total, witness| {
                total.checked_add(witness.expected_length)
            });
        if self
            .retained_file_witnesses
            .len()
            .checked_add(new_objects)
            .is_none_or(|count| count > AUTHORITATIVE_STORE_MAX_OBJECTS)
            || total
                .zip(added_bytes)
                .and_then(|(total, added)| total.checked_add(added))
                .is_none_or(|total| total > AUTHORITATIVE_STORE_MAX_NAMESPACE_BYTES)
        {
            return Err(StoreProvisioningError::ResourceLimit);
        }
        Ok(())
    }

    // Requires the caller's publication lock to remain live through readback.
    fn publish_registered_policy(
        &mut self,
        id: RecordId,
        bytes: Vec<u8>,
        identities: Vec<RecordId>,
    ) -> Result<JournalReference, StoreProvisioningError> {
        self.provisioning_record_budget(id, &bytes, None)?;
        let head = self.retained_journal.current_head_reference();
        let index = head
            .entry_index()
            .value()
            .checked_add(1)
            .and_then(|value| JournalEntryIndex::try_from(value).ok())
            .ok_or(StoreProvisioningError::JournalConstruction)?;
        let context = self
            .retained_journal
            .resolve_reference(&head)
            .map_err(|_| StoreProvisioningError::ReplayMismatch)?;
        let identities = IdentityDependencyCollection::from_unordered_semantic_elements(
            identities
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .map(IdentityDependency::record_id)
                .collect(),
        )
        .map_err(|_| StoreProvisioningError::JournalConstruction)?;
        let authorities = AuthorityDependencyCollection::from_unordered_semantic_elements(
            AuthorityDependencyContext::new(head.registry_id(), index),
            vec![],
        )
        .map_err(|_| StoreProvisioningError::JournalConstruction)?;
        let mut entry = vec![0x82, 0x78, 0x20];
        entry.extend_from_slice(JOURNAL_ENTRY_DOMAIN);
        entry.extend_from_slice(&[0xac, 0, 1, 1]);
        encode_bstr_32(&mut entry, head.registry_id().as_bytes());
        entry.push(2);
        encode_uint(&mut entry, index.value());
        entry.push(3);
        encode_bstr_32(&mut entry, head.entry_hash().as_bytes());
        entry.push(4);
        encode_uint(&mut entry, 400);
        entry.push(5);
        encode_bstr_32(&mut entry, id.as_bytes());
        entry.push(6);
        entry.extend_from_slice(&identities.authoritative_cbor());
        entry.push(7);
        entry.extend_from_slice(&authorities.authoritative_cbor());
        entry.extend_from_slice(&[8, 7, 9]);
        encode_bstr_32(&mut entry, id.as_bytes());
        entry.push(10);
        encode_bstr_32(&mut entry, context.storage_capability_class_id().as_bytes());
        entry.push(11);
        encode_bstr_32(&mut entry, context.environment_observation_id().as_bytes());
        let reference = JournalReference::new(
            head.registry_id(),
            index,
            JournalEntryHash::try_from(Sha256::digest(&entry).as_slice()).expect("SHA-256 width"),
            EventTypeId::try_from(400).expect("assigned event"),
            EventRecordId::try_from(id.as_bytes().as_slice()).expect("RecordId width"),
        );
        // Reserve the complete publication, including the next slot, before
        // either object can become visible or make cold reopening impossible.
        self.provisioning_record_budget(id, &bytes, Some(&entry))?;
        publish_record_bytes(self, id, &bytes)
            .map_err(|_| StoreProvisioningError::RecordPublication)?;
        self.revalidate_retained_generation()
            .map_err(|_| StoreProvisioningError::RetainedGenerationChanged)?;
        let directory = self.root.join("journal");
        let hold = self
            .acquire_publication_directory_hold(2, "journal")
            .map_err(|_| StoreProvisioningError::RetainedGenerationChanged)?;
        let contents = namespace_contents_path(&directory, &hold)
            .map_err(|_| StoreProvisioningError::RetainedGenerationChanged)?;
        match publish_journal_slot(
            &contents,
            &hold,
            Path::new(&format!("{:020}.cbor", index.value())),
            &entry,
        )
        .map_err(|_| StoreProvisioningError::JournalPublication)?
        {
            JournalSlotPublication::Published(_) => {}
            JournalSlotPublication::Conflict => {
                return Err(StoreProvisioningError::JournalConflict)
            }
            JournalSlotPublication::VisibleReceiptUncertain => {
                return Err(StoreProvisioningError::JournalPublication)
            }
        }
        self.reload_authoritative_namespaces().map_err(map_reload)?;
        if self.retained_journal.current_head_reference() != reference
            || self.resolve(id) != Some(bytes.as_slice())
        {
            return Err(StoreProvisioningError::ReplayMismatch);
        }
        self.retained_journal
            .resolve_reference(&reference)
            .map_err(|_| StoreProvisioningError::ReplayMismatch)?;
        Ok(reference)
    }
}

fn map_reload(error: AuthoritativeReviewAdmissionAcceptanceError) -> StoreProvisioningError {
    match error {
        AuthoritativeReviewAdmissionAcceptanceError::Store(error) => {
            StoreProvisioningError::Store(error)
        }
        _ => StoreProvisioningError::RetainedGenerationChanged,
    }
}

fn validate_check_members(
    resolver: &impl ExactRecordByteResolver,
    checks: &CheckSetRecord,
) -> Result<(), RecordDecodeError> {
    for id in checks.check_refs() {
        let record =
            CheckRecord::decode_authoritative(resolver.resolve(*id).ok_or(RecordDecodeError)?)?;
        if record.record_id() != *id {
            return Err(RecordDecodeError);
        }
    }
    Ok(())
}

pub(super) fn validate_review_parameters(
    resolver: &impl ExactRecordByteResolver,
    scope: RecordId,
    method: RecordId,
    checks: RecordId,
) -> Result<(), RecordDecodeError> {
    let scope_record =
        ScopeRecord::decode_authoritative(resolver.resolve(scope).ok_or(RecordDecodeError)?)?;
    let method_record =
        MethodRecord::decode_authoritative(resolver.resolve(method).ok_or(RecordDecodeError)?)?;
    let checks_record =
        CheckSetRecord::decode_authoritative(resolver.resolve(checks).ok_or(RecordDecodeError)?)?;
    if scope_record.record_id() != scope
        || method_record.record_id() != method
        || checks_record.record_id() != checks
    {
        return Err(RecordDecodeError);
    }
    validate_check_members(resolver, &checks_record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(windows, target_os = "linux"))]
    fn provisioning_guard_holds_publication_lock_until_dropped() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!(
                "provisioning-guard-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        let mut store = AuthoritativeRegistryStore::initialize_selected_profile(
            &root,
            RegistryId([0x7a; 32]),
            "provisioning-guard-test",
        )
        .unwrap();
        let guard = store.provisioning_lock().unwrap();
        // The owned guard must permit Store mutation while excluding a second
        // acquisition, then release the publication lock on drop.
        store.reload_authoritative_namespaces().unwrap();
        assert!(matches!(
            store.provisioning_lock(),
            Err(StoreProvisioningError::PublicationLockUnavailable)
        ));
        drop(guard);
        let next_guard = store.provisioning_lock().unwrap();
        drop(next_guard);
        drop(store);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn request_replay_independently_resolves_every_check_member() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!("request-parameter-replay-{}", std::process::id()));
        let source = root.with_extension("source");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("a"), b"request parameter replay").unwrap();
        let mut store = AuthoritativeRegistryStore::initialize_selected_profile(
            &root,
            RegistryId([0x79; 32]),
            "request-replay-test",
        )
        .unwrap();
        let scope = |profile| {
            ScopeRecord::new(ScopeRecordInput {
                scope_profile_id: profile,
                scope_profile_version: 1,
                scope_payload: vec![],
                scope_label: None,
            })
            .unwrap()
        };
        let pf_scope = scope(2);
        let pr_scope = scope(1);
        store.stage_scope_record(&pf_scope).unwrap();
        store.stage_scope_record(&pr_scope).unwrap();
        let method = MethodRecord::new(MethodRecordInput {
            method_profile_id: 1,
            method_profile_version: 1,
            method_payload: vec![],
            method_label: None,
        })
        .unwrap();
        let check = CheckRecord::new(CheckRecordInput {
            check_profile_id: 1,
            check_profile_version: 1,
            check_payload: vec![],
            check_label: None,
        })
        .unwrap();
        store.stage_method_record(&method).unwrap();
        store.stage_check_record(&check).unwrap();
        let checks = CheckSetRecord::new(CheckSetRecordInput {
            check_refs: vec![check.record_id()],
        })
        .unwrap();
        store.stage_check_set_record(&checks).unwrap();
        let pf = store
            .register_selected_minimal_policy(pf_scope.record_id())
            .unwrap();
        let pr = store
            .register_selected_review_policy(
                pr_scope.record_id(),
                vec![ReviewAdmissionReviewRequirement::new(
                    1,
                    pr_scope.record_id(),
                    method.record_id(),
                    checks.record_id(),
                    1,
                )
                .unwrap()],
                None,
                None,
                None,
            )
            .unwrap();
        let prepared = store
            .prepare_selected_embedded_freeze(SelectedEmbeddedFreezePreparationInput {
                source_root: source.clone(),
                freeze_attempt_id: FreezeAttemptId([0x78; 32]),
                policy_record_id: record_id_from_event_reference(&pf),
            })
            .unwrap();
        let freeze = store
            .commit_prepared_selected_embedded_freeze(prepared)
            .unwrap();
        let recorded = store
            .record_selected_review_request(SelectedReviewRequestInput {
                freeze_authority: freeze,
                review_policy_record_id: record_id_from_event_reference(&pr),
                review_role_id: 1,
            })
            .unwrap();
        let entry = store.retained_journal.entries.last().unwrap();
        assert!(validate_review_request_replay_contract(
            &store.retained_journal,
            entry,
            recorded.request(),
            &store.records
        )
        .is_ok());
        // Exercise Request's validator independently of the earlier event-400
        // validator: a cached Policy witness is not a resolved CHECK closure.
        store.records.retain(|(id, _)| *id != check.record_id());
        let structural = validate_review_request_replay_contract(
            &store.retained_journal,
            entry,
            recorded.request(),
            &store.records,
        );
        let selected = store.validate_selected_review_request_from_retained_snapshot(
            recorded.request_event_reference().clone(),
            &mut vec![],
            &mut 0,
            u64::MAX,
        );
        drop(store);
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(source).unwrap();
        assert!(
            structural.is_err(),
            "Request replay accepted missing CHECK member"
        );
        assert!(
            selected.is_err(),
            "selected Request witness accepted missing CHECK member"
        );
    }
}
