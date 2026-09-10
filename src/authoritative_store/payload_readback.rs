use super::*;

/// Bounded, read-only bytes returned from the retained-handle selected payload path.
/// Each byte vector corresponds to the same-index artifact in `manifest()`. Bytes
/// matched that Manifest when read; this is not Agent attribution, an atomic
/// snapshot, a new capture publication receipt, or a live execution witness.
#[derive(Debug)]
pub struct SelectedEmbeddedPayload {
    freeze_authority: AuthoritativeFreezeCommittedBinding,
    manifest: ManifestRecord,
    bytes: Vec<Vec<u8>>,
}
impl SelectedEmbeddedPayload {
    pub fn freeze_authority(&self) -> &AuthoritativeFreezeCommittedBinding {
        &self.freeze_authority
    }
    pub fn manifest(&self) -> &ManifestRecord {
        &self.manifest
    }
    pub fn artifact_bytes(&self) -> &[Vec<u8>] {
        &self.bytes
    }
}
impl AuthoritativeRegistryStore {
    /// Revalidate exact selected event-101 authority and return its complete
    /// Manifest-matched payload through Store-owned handles. No caller pathname,
    /// artifact digest, or stale authority witness substitutes for resolution.
    /// Applies the existing Store intake bounds; does not write or refresh history.
    pub fn read_selected_embedded_payload(
        &self,
        committed_event_reference: JournalReference,
    ) -> Result<SelectedEmbeddedPayload, AuthoritativeFreezeCommittedBindingError> {
        let freeze_authority =
            self.validate_freeze_committed_authority(committed_event_reference)?;
        let manifest = self
            .resolve(freeze_authority.manifest_record_id())
            .and_then(|bytes| ManifestRecord::decode_authoritative(bytes).ok())
            .filter(|manifest| manifest.record_id() == freeze_authority.manifest_record_id())
            .ok_or(AuthoritativeFreezeCommittedBindingError::SelectedPrerequisiteInvalid)?;
        // Return the actual bytes from the validating traversal, never validate
        // first then reread an ambient payload path in the consumer.
        let source = read_validated_selected_embedded_payload(
            self,
            freeze_authority.freeze_attempt_id(),
            &manifest,
        )
        .map_err(|error| match error {
            SelectedEmbeddedFreezePreparationError::SourceResourceLimit => {
                AuthoritativeFreezeCommittedBindingError::ResourceLimit
            }
            _ => AuthoritativeFreezeCommittedBindingError::SelectedPrerequisiteInvalid,
        })?;
        self.revalidate_retained_generation()
            .map_err(|()| AuthoritativeFreezeCommittedBindingError::RetainedGenerationChanged)?;
        Ok(SelectedEmbeddedPayload {
            freeze_authority,
            manifest,
            bytes: source.bytes,
        })
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn nested_payload_directory_flush_requires_publication_access() {
        let root = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!("payload-flush-access-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::create_dir(root.join("child")).unwrap();
        let parent = open_publication_directory_hold(&root).unwrap();
        let read = open_child_directory_hold(&parent, &root, "child").unwrap();
        let error = read.sync_all().unwrap_err();
        assert_eq!(
            error.raw_os_error(),
            Some(5),
            "actual Windows read-only directory flush"
        );
        let publish =
            open_publication_child_directory_hold(&parent, &root, &read, "child").unwrap();
        sync_retained_directory(&root.join("child"), &publish).unwrap();
        drop(publish);
        drop(read);
        drop(parent);
        fs::remove_dir_all(root).unwrap();
    }
}
