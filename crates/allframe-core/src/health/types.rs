//! Health check types and data structures

use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Status of an individual dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyStatus {
    /// The dependency is functioning normally
    Healthy,
    /// The dependency is functioning but with reduced capacity or performance
    Degraded(String),
    /// The dependency is not functioning
    Unhealthy(String),
}

impl DependencyStatus {
    /// Check if the status is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, DependencyStatus::Healthy)
    }

    /// Check if the status is degraded
    pub fn is_degraded(&self) -> bool {
        matches!(self, DependencyStatus::Degraded(_))
    }

    /// Check if the status is unhealthy
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, DependencyStatus::Unhealthy(_))
    }

    /// Get the message if degraded or unhealthy
    pub fn message(&self) -> Option<&str> {
        match self {
            DependencyStatus::Healthy => None,
            DependencyStatus::Degraded(msg) | DependencyStatus::Unhealthy(msg) => Some(msg),
        }
    }
}

/// Overall health status of the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverallStatus {
    /// All dependencies are healthy
    Healthy,
    /// Some non-critical dependencies are unhealthy or degraded
    Degraded,
    /// One or more critical dependencies are unhealthy
    Unhealthy,
}

impl OverallStatus {
    /// Convert to HTTP status code
    pub fn http_status_code(&self) -> u16 {
        match self {
            OverallStatus::Healthy => 200,
            OverallStatus::Degraded => 200, // Still serving, but degraded
            OverallStatus::Unhealthy => 503,
        }
    }

    /// Convert to a simple string for logging
    pub fn as_str(&self) -> &'static str {
        match self {
            OverallStatus::Healthy => "healthy",
            OverallStatus::Degraded => "degraded",
            OverallStatus::Unhealthy => "unhealthy",
        }
    }
}

/// Report for an individual dependency check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyReport {
    /// Name of the dependency
    pub name: String,
    /// Current status
    pub status: DependencyStatus,
    /// Time taken to check this dependency
    #[serde(with = "duration_millis")]
    pub duration: Duration,
    /// Whether this dependency is critical
    pub critical: bool,
}

/// Complete health report for the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall health status
    pub status: OverallStatus,
    /// Individual dependency reports
    pub dependencies: Vec<DependencyReport>,
    /// Total time taken to check all dependencies
    #[serde(with = "duration_millis")]
    pub total_duration: Duration,
    /// Timestamp of this report
    #[serde(with = "system_time_rfc3339")]
    pub timestamp: SystemTime,
}

impl HealthReport {
    /// Get the HTTP status code for this report
    pub fn http_status_code(&self) -> u16 {
        self.status.http_status_code()
    }

    /// Check if all dependencies are healthy
    pub fn is_healthy(&self) -> bool {
        self.status == OverallStatus::Healthy
    }

    /// Get failed dependencies (unhealthy)
    pub fn failed_dependencies(&self) -> Vec<&DependencyReport> {
        self.dependencies
            .iter()
            .filter(|d| d.status.is_unhealthy())
            .collect()
    }

    /// Get degraded dependencies
    pub fn degraded_dependencies(&self) -> Vec<&DependencyReport> {
        self.dependencies
            .iter()
            .filter(|d| d.status.is_degraded())
            .collect()
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Serde helper for Duration as milliseconds
mod duration_millis {
    use std::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Serde helper for SystemTime as Unix timestamp (seconds)
mod system_time_rfc3339 {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_status_checks() {
        assert!(DependencyStatus::Healthy.is_healthy());
        assert!(!DependencyStatus::Healthy.is_degraded());
        assert!(!DependencyStatus::Healthy.is_unhealthy());

        let degraded = DependencyStatus::Degraded("slow".into());
        assert!(!degraded.is_healthy());
        assert!(degraded.is_degraded());
        assert!(!degraded.is_unhealthy());

        let unhealthy = DependencyStatus::Unhealthy("down".into());
        assert!(!unhealthy.is_healthy());
        assert!(!unhealthy.is_degraded());
        assert!(unhealthy.is_unhealthy());
    }

    #[test]
    fn test_dependency_status_message() {
        assert_eq!(DependencyStatus::Healthy.message(), None);
        assert_eq!(
            DependencyStatus::Degraded("slow".into()).message(),
            Some("slow")
        );
        assert_eq!(
            DependencyStatus::Unhealthy("down".into()).message(),
            Some("down")
        );
    }

    #[test]
    fn test_overall_status_http_codes() {
        assert_eq!(OverallStatus::Healthy.http_status_code(), 200);
        assert_eq!(OverallStatus::Degraded.http_status_code(), 200);
        assert_eq!(OverallStatus::Unhealthy.http_status_code(), 503);
    }

    #[test]
    fn test_overall_status_as_str() {
        assert_eq!(OverallStatus::Healthy.as_str(), "healthy");
        assert_eq!(OverallStatus::Degraded.as_str(), "degraded");
        assert_eq!(OverallStatus::Unhealthy.as_str(), "unhealthy");
    }

    #[test]
    fn test_health_report_json() {
        let report = HealthReport {
            status: OverallStatus::Healthy,
            dependencies: vec![DependencyReport {
                name: "test".into(),
                status: DependencyStatus::Healthy,
                duration: Duration::from_millis(10),
                critical: true,
            }],
            total_duration: Duration::from_millis(15),
            timestamp: SystemTime::now(),
        };

        let json = report.to_json().unwrap();
        assert!(json.contains("\"status\": \"healthy\""));
        assert!(json.contains("\"name\": \"test\""));
    }
}
