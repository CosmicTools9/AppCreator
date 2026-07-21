//! Generation metrics collection
//!
//! Tracks code generation performance including duration, entity count, file count, and errors.

use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Duration histogram buckets (in seconds)
const DURATION_BUCKETS: &[f64] = &[0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0];

/// Generation type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationType {
    /// Full generation
    Full,
    /// Incremental generation
    Incremental,
    /// Preview generation
    Preview,
}

impl GenerationType {
    /// Convert to string label
    pub fn as_str(&self) -> &'static str {
        match self {
            GenerationType::Full => "full",
            GenerationType::Incremental => "incremental",
            GenerationType::Preview => "preview",
        }
    }
}

/// Generation metrics collector
pub struct GenerationMetrics {
    registry: Registry,
    duration_seconds: HistogramVec,
    entities_total: IntCounterVec,
    files_total: IntCounterVec,
    errors_total: IntCounterVec,
}

impl GenerationMetrics {
    /// Create new generation metrics
    pub fn new() -> Self {
        let registry = Registry::new();

        let duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "generation_duration_seconds",
                "Time spent generating code in seconds",
            )
            .buckets(DURATION_BUCKETS.to_vec()),
            &["type"],
        )
        .expect("Failed to create duration histogram");

        let entities_total = IntCounterVec::new(
            Opts::new(
                "generation_entities_total",
                "Total number of entities processed",
            ),
            &["type"],
        )
        .expect("Failed to create entities counter");

        let files_total = IntCounterVec::new(
            Opts::new("generation_files_total", "Total number of files generated"),
            &["type"],
        )
        .expect("Failed to create files counter");

        let errors_total = IntCounterVec::new(
            Opts::new(
                "generation_errors_total",
                "Total number of generation errors",
            ),
            &["type"],
        )
        .expect("Failed to create errors counter");

        registry
            .register(Box::new(duration_seconds.clone()))
            .expect("Failed to register duration");
        registry
            .register(Box::new(entities_total.clone()))
            .expect("Failed to register entities");
        registry
            .register(Box::new(files_total.clone()))
            .expect("Failed to register files");
        registry
            .register(Box::new(errors_total.clone()))
            .expect("Failed to register errors");

        Self {
            registry,
            duration_seconds,
            entities_total,
            files_total,
            errors_total,
        }
    }
}

impl Default for GenerationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl GenerationMetrics {
    /// Record a generation event
    pub fn record(
        &self,
        generation_type: GenerationType,
        duration: Duration,
        entity_count: usize,
        c_file_count: usize,
        error_count: usize,
    ) {
        let type_label = generation_type.as_str();

        self.duration_seconds
            .with_label_values(&[type_label])
            .observe(duration.as_secs_f64());

        self.entities_total
            .with_label_values(&[type_label])
            .inc_by(entity_count as u64);

        self.files_total
            .with_label_values(&[type_label])
            .inc_by(c_file_count as u64);

        self.errors_total
            .with_label_values(&[type_label])
            .inc_by(error_count as u64);

        // Warn if slow operation (>5 seconds)
        if duration.as_secs() > 5 {
            common::telemetry::warn!(
                "Slow generation operation: type={} duration={:.2}s entities={} files={}",
                type_label,
                duration.as_secs_f64(),
                entity_count,
                c_file_count
            );
        }
    }

    /// Gather all metrics families
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}

/// Global metrics instance
static METRICS: RwLock<Option<Arc<GenerationMetrics>>> = RwLock::new(None);

/// Initialize the global metrics instance
pub fn init_metrics() {
    let metrics = Arc::new(GenerationMetrics::new());
    if let Ok(mut guard) = METRICS.write() {
        *guard = Some(metrics);
    }
}

/// Record a generation event
pub fn record_generation(
    generation_type: GenerationType,
    duration: Duration,
    entity_count: usize,
    c_file_count: usize,
    error_count: usize,
) {
    if let Ok(guard) = METRICS.read() {
        if let Some(metrics) = guard.as_ref() {
            metrics.record(
                generation_type,
                duration,
                entity_count,
                c_file_count,
                error_count,
            );
            return;
        }
    }
    common::telemetry::warn!("Generation metrics not initialized");
}

/// Generation timer for measuring duration
pub struct GenerationTimer {
    start: Instant,
    generation_type: GenerationType,
}

impl GenerationTimer {
    /// Start a new timer for the given generation type
    pub fn new(generation_type: GenerationType) -> Self {
        Self {
            start: Instant::now(),
            generation_type,
        }
    }

    /// Finish timing and record the metrics
    pub fn finish(self, entity_count: usize, c_file_count: usize, error_count: usize) {
        let duration = self.start.elapsed();
        record_generation(
            self.generation_type,
            duration,
            entity_count,
            c_file_count,
            error_count,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_type_as_str() {
        assert_eq!(GenerationType::Full.as_str(), "full");
        assert_eq!(GenerationType::Incremental.as_str(), "incremental");
        assert_eq!(GenerationType::Preview.as_str(), "preview");
    }

    #[test]
    fn test_generation_metrics_new() {
        let metrics = GenerationMetrics::new();
        // Record some data so metrics appear in gather()
        metrics.record(GenerationType::Full, Duration::from_secs(1), 10, 5, 0);
        let families = metrics.gather();
        // Should have 4 metric families
        assert_eq!(families.len(), 4);
    }

    #[test]
    fn test_generation_timer() {
        let timer = GenerationTimer::new(GenerationType::Full);
        std::thread::sleep(Duration::from_millis(10));
        timer.finish(10, 5, 0);
        // Test passes if no panic
    }

    #[test]
    fn test_init_and_record() {
        init_metrics();
        record_generation(GenerationType::Full, Duration::from_secs(1), 10, 5, 0);
    }
}
