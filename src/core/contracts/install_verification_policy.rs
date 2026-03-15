//! Install verification policy.

use serde::{Deserialize, Serialize};

/// Policy for verifying module installations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallVerificationPolicy {
    /// Whether to verify signatures.
    pub verify_signature: bool,
    /// Whether to verify checksums.
    pub verify_checksum: bool,
    /// Whether to run post-install tests.
    pub run_tests: bool,
}

impl Default for InstallVerificationPolicy {
    fn default() -> Self {
        Self {
            verify_signature: true,
            verify_checksum: true,
            run_tests: false,
        }
    }
}
