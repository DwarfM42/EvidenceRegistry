//! Non-authoritative managed execution companion; Core owns evidence authority.
mod submission;
pub use submission::{
    approved_artifact_name, parse_submission, RequestBinding, Submission, SubmissionError,
    MAX_SUBMISSION_BYTES,
};
pub mod bridge;
pub mod inspection;
pub mod ledger;
pub mod process;
