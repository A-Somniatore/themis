//! Semantic versioning support for contracts.
//!
//! Implements semantic versioning (SemVer) for contract version management.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use crate::error::ThemisError;

/// A semantic version (major.minor.patch).
///
/// Follows the [Semantic Versioning 2.0.0](https://semver.org/) specification.
///
/// # Example
///
/// ```rust
/// use themis_core::Version;
///
/// let v1 = Version::new(1, 0, 0);
/// let v2 = Version::new(1, 1, 0);
///
/// assert!(v2 > v1);
/// assert!(v2.is_compatible_with(&v1));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    /// Major version - incremented for breaking changes
    pub major: u32,
    /// Minor version - incremented for backward-compatible additions
    pub minor: u32,
    /// Patch version - incremented for backward-compatible fixes
    pub patch: u32,
    /// Optional pre-release identifier (e.g., "alpha", "beta.1")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_release: Option<String>,
}

impl Version {
    /// Creates a new version with the given major, minor, and patch numbers.
    #[must_use]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    /// Creates a new version with a pre-release identifier.
    #[must_use]
    pub fn with_pre_release(
        major: u32,
        minor: u32,
        patch: u32,
        pre_release: impl Into<String>,
    ) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release.into()),
        }
    }

    /// Returns true if this version is compatible with the other version.
    ///
    /// Two versions are compatible if they have the same major version and
    /// this version is greater than or equal to the other.
    ///
    /// # Example
    ///
    /// ```rust
    /// use themis_core::Version;
    ///
    /// let v1_0 = Version::new(1, 0, 0);
    /// let v1_1 = Version::new(1, 1, 0);
    /// let v2_0 = Version::new(2, 0, 0);
    ///
    /// assert!(v1_1.is_compatible_with(&v1_0));
    /// assert!(!v1_0.is_compatible_with(&v1_1)); // older can't satisfy newer
    /// assert!(!v2_0.is_compatible_with(&v1_0)); // different major
    /// ```
    #[must_use]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major && self >= other
    }

    /// Returns true if this version represents a breaking change from the other.
    ///
    /// A version is a breaking change if the major version is different.
    #[must_use]
    pub fn is_breaking_from(&self, other: &Self) -> bool {
        self.major != other.major
    }

    /// Returns true if this is a pre-release version.
    #[must_use]
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }

    /// Returns the next major version (resets minor and patch to 0).
    #[must_use]
    pub fn next_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    /// Returns the next minor version (resets patch to 0).
    #[must_use]
    pub fn next_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    /// Returns the next patch version.
    #[must_use]
    pub fn next_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = ThemisError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Split off pre-release if present
        let (version_part, pre_release) = match s.split_once('-') {
            Some((v, pre)) => (v, Some(pre.to_string())),
            None => (s, None),
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(ThemisError::InvalidVersion {
                version: s.to_string(),
                reason: "Version must have exactly 3 parts (major.minor.patch)".to_string(),
            });
        }

        let major = parts[0].parse().map_err(|_| ThemisError::InvalidVersion {
            version: s.to_string(),
            reason: "Invalid major version number".to_string(),
        })?;

        let minor = parts[1].parse().map_err(|_| ThemisError::InvalidVersion {
            version: s.to_string(),
            reason: "Invalid minor version number".to_string(),
        })?;

        let patch = parts[2].parse().map_err(|_| ThemisError::InvalidVersion {
            version: s.to_string(),
            reason: "Invalid patch version number".to_string(),
        })?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }
        // Pre-release versions have lower precedence than normal versions
        match (&self.pre_release, &other.pre_release) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_new() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre_release.is_none());
    }

    #[test]
    fn test_version_display() {
        assert_eq!(Version::new(1, 0, 0).to_string(), "1.0.0");
        assert_eq!(Version::new(2, 10, 5).to_string(), "2.10.5");
        assert_eq!(
            Version::with_pre_release(1, 0, 0, "alpha").to_string(),
            "1.0.0-alpha"
        );
    }

    #[test]
    fn test_version_parse() {
        assert_eq!("1.0.0".parse::<Version>().unwrap(), Version::new(1, 0, 0));
        assert_eq!("2.10.5".parse::<Version>().unwrap(), Version::new(2, 10, 5));
        assert_eq!(
            "1.0.0-beta.1".parse::<Version>().unwrap(),
            Version::with_pre_release(1, 0, 0, "beta.1")
        );
    }

    #[test]
    fn test_version_parse_invalid() {
        assert!("1.0".parse::<Version>().is_err());
        assert!("1.0.0.0".parse::<Version>().is_err());
        assert!("a.b.c".parse::<Version>().is_err());
    }

    #[test]
    fn test_version_ordering() {
        let v1_0_0 = Version::new(1, 0, 0);
        let v1_0_1 = Version::new(1, 0, 1);
        let v1_1_0 = Version::new(1, 1, 0);
        let v2_0_0 = Version::new(2, 0, 0);

        assert!(v1_0_0 < v1_0_1);
        assert!(v1_0_1 < v1_1_0);
        assert!(v1_1_0 < v2_0_0);
    }

    #[test]
    fn test_version_pre_release_ordering() {
        let v1_0_0_alpha = Version::with_pre_release(1, 0, 0, "alpha");
        let v1_0_0 = Version::new(1, 0, 0);

        assert!(v1_0_0_alpha < v1_0_0);
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0 = Version::new(1, 0, 0);
        let v1_1 = Version::new(1, 1, 0);
        let v2_0 = Version::new(2, 0, 0);

        assert!(v1_1.is_compatible_with(&v1_0));
        assert!(!v1_0.is_compatible_with(&v1_1));
        assert!(!v2_0.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_version_breaking() {
        let v1_0 = Version::new(1, 0, 0);
        let v1_1 = Version::new(1, 1, 0);
        let v2_0 = Version::new(2, 0, 0);

        assert!(!v1_1.is_breaking_from(&v1_0));
        assert!(v2_0.is_breaking_from(&v1_0));
    }

    #[test]
    fn test_version_next() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.next_patch(), Version::new(1, 2, 4));
        assert_eq!(v.next_minor(), Version::new(1, 3, 0));
        assert_eq!(v.next_major(), Version::new(2, 0, 0));
    }

    #[test]
    fn test_version_serialization() {
        let v = Version::new(1, 2, 3);
        let json = serde_json::to_string(&v).unwrap();
        let deserialized: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, deserialized);
    }
}
