//! Generation Progress Events
//!
//! Event types for WebSocket real-time generation progress updates.

use serde::{Deserialize, Serialize};

/// Generation job events sent via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum GenerationEvent {
    /// Generation job started
    Started {
        job_id: String,
        app_name: String,
        started_at: chrono::DateTime<chrono::Utc>,
    },

    /// Progress update during generation
    Progress {
        job_id: String,
        phase: String,
        progress_percent: u8,
        message: String,
    },

    /// Generation completed successfully
    Completed {
        job_id: String,
        app_name: String,
        output_path: String,
        completed_at: chrono::DateTime<chrono::Utc>,
    },

    /// Generation failed with error
    Failed {
        job_id: String,
        error: String,
        failed_at: chrono::DateTime<chrono::Utc>,
    },

    /// Control action (cancel, pause, resume)
    Control { job_id: String, action: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_event_serialization() {
        let event = GenerationEvent::Started {
            job_id: "test-job".to_string(),
            app_name: "test-app".to_string(),
            started_at: chrono::Utc::now(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "started");
        assert_eq!(json["data"]["job_id"], "test-job");
    }

    #[test]
    fn test_generation_event_deserialization() {
        let json = serde_json::json!({
            "type": "completed",
            "data": {
                "job_id": "job-1",
                "app_name": "my-app",
                "output_path": "/tmp/my-app",
                "completed_at": "2026-01-01T00:00:00Z"
            }
        });

        let event: GenerationEvent = serde_json::from_value(json).unwrap();
        match event {
            GenerationEvent::Completed { app_name, .. } => {
                assert_eq!(app_name, "my-app");
            }
            _ => panic!("Expected Completed variant"),
        }
    }

    #[test]
    fn test_control_event() {
        let event = GenerationEvent::Control {
            job_id: "job-1".to_string(),
            action: "cancel".to_string(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["type"], "control");
        assert_eq!(json["data"]["action"], "cancel");
    }
}
