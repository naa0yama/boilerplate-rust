//! `OTel` semconv-compliant process metrics (feature `process-metrics`).
//!
//! All metric values come from the cross-platform [`sysinfo`] crate; no
//! per-OS shims or `/proc` parsing are used. Metric names follow the
//! [OpenTelemetry semantic conventions for process metrics][semconv].
//!
//! [semconv]: https://opentelemetry.io/docs/specs/semconv/system/process-metrics/

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use opentelemetry::KeyValue;
use opentelemetry::metrics::{Meter, ObservableCounter, ObservableGauge, ObservableUpDownCounter};
use opentelemetry_semantic_conventions::{attribute, metric as semconv};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

/// Handles for all process metric observable instruments.
///
/// Dropping this struct de-registers all observable callbacks. Keep it alive
/// for the duration of the program (typically as a field of `Meters`).
/// Create exactly once after `opentelemetry::global::set_meter_provider` has
/// been called.
pub struct ProcessMetricHandles {
    _cpu_time: ObservableCounter<f64>,
    _cpu_utilization: ObservableGauge<f64>,
    _mem_usage: ObservableUpDownCounter<i64>,
    _mem_virtual: ObservableUpDownCounter<i64>,
    _disk_io: ObservableCounter<u64>,
    _thread_count: ObservableUpDownCounter<i64>,
    _open_fds: ObservableUpDownCounter<i64>,
    _uptime: ObservableGauge<f64>,
}

impl std::fmt::Debug for ProcessMetricHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessMetricHandles")
            .finish_non_exhaustive()
    }
}

impl ProcessMetricHandles {
    /// Register all process metric instruments against `meter`.
    ///
    /// Each instrument polls the OS via `sysinfo` on every collection cycle
    /// (default: every 5 s via `PeriodicReader`). A shared `Arc<Mutex<System>>`
    /// is used to avoid redundant refresh calls; `try_lock` prevents blocking
    /// if two callbacks race.
    #[must_use]
    pub fn register(meter: &Meter) -> Self {
        let pid = current_pid();
        let cpu_count = logical_cpu_count();
        let system = Arc::new(Mutex::new(System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        )));

        Self {
            _cpu_time: register_cpu_time(meter, Arc::clone(&system), pid),
            _cpu_utilization: register_cpu_utilization(meter, Arc::clone(&system), pid, cpu_count),
            _mem_usage: register_mem_usage(meter, Arc::clone(&system), pid),
            _mem_virtual: register_mem_virtual(meter, Arc::clone(&system), pid),
            _disk_io: register_disk_io(meter, Arc::clone(&system), pid),
            _thread_count: register_thread_count(meter, Arc::clone(&system), pid),
            _open_fds: register_open_fds(meter, Arc::clone(&system), pid),
            _uptime: register_uptime(meter, system, pid),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
// u32 → usize: PIDs always fit (usize ≥ 32 bits on every supported target)
fn current_pid() -> Pid {
    Pid::from(std::process::id() as usize)
}

/// Return the number of logical CPUs available to this process.
///
/// Used to normalise `sysinfo::cpu_usage()` (which can exceed 100 % on
/// multi-core hosts) into the 0..1 fraction required by
/// `process.cpu.utilization`.
fn logical_cpu_count() -> u32 {
    let n = std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1);
    u32::try_from(n).unwrap_or(1)
}

/// Refresh the target process and call `f` with its snapshot.
///
/// Uses `try_lock` so that a busy lock on the metric export thread is skipped
/// rather than blocked — a single missed sample is acceptable. Returns `None`
/// if the lock is contended or the process is not found.
fn with_process<F, T>(system: &Arc<Mutex<System>>, pid: Pid, f: F) -> Option<T>
where
    F: FnOnce(&sysinfo::Process) -> T,
{
    let mut sys = system.try_lock().ok()?;
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        false,
        ProcessRefreshKind::everything(),
    );
    sys.process(pid).map(f)
}

// ---------------------------------------------------------------------------
// Individual instrument registrations
// ---------------------------------------------------------------------------

fn register_cpu_time(
    meter: &Meter,
    system: Arc<Mutex<System>>,
    pid: Pid,
) -> ObservableCounter<f64> {
    meter
        .f64_observable_counter(semconv::PROCESS_CPU_TIME)
        .with_unit("s")
        .with_description(
            "Accumulated CPU time consumed by this process (user + system combined). \
             The cpu.mode attribute is omitted because sysinfo does not expose a \
             user/system split cross-platform.",
        )
        .with_callback(move |obs| {
            if let Some(ms) = with_process(&system, pid, sysinfo::Process::accumulated_cpu_time) {
                // accumulated_cpu_time is u64 ms. f64 can represent values up to
                // ~9 × 10^15 ms (~285 million years) without precision loss — safe
                // for any realistic process lifetime.
                #[allow(clippy::as_conversions, clippy::cast_precision_loss)]
                let cpu_secs = ms as f64 / 1000.0;
                obs.observe(cpu_secs, &[]);
            }
        })
        .build()
}

fn register_cpu_utilization(
    meter: &Meter,
    system: Arc<Mutex<System>>,
    pid: Pid,
    cpu_count: u32,
) -> ObservableGauge<f64> {
    meter
        .f64_observable_gauge(semconv::PROCESS_CPU_UTILIZATION)
        .with_unit("1")
        .with_description(
            "Recent CPU utilization fraction for this process \
             (sysinfo cpu_usage / logical_cpu_count / 100). \
             The first sample after start is typically 0.",
        )
        .with_callback(move |obs| {
            if let Some(pct) = with_process(&system, pid, sysinfo::Process::cpu_usage) {
                let utilization = f64::from(pct) / (f64::from(cpu_count) * 100.0);
                obs.observe(utilization, &[]);
            }
        })
        .build()
}

fn register_mem_usage(
    meter: &Meter,
    system: Arc<Mutex<System>>,
    pid: Pid,
) -> ObservableUpDownCounter<i64> {
    meter
        .i64_observable_up_down_counter(semconv::PROCESS_MEMORY_USAGE)
        .with_unit("By")
        .with_description("Resident set size (RSS) of this process in bytes")
        .with_callback(move |obs| {
            if let Some(bytes) = with_process(&system, pid, sysinfo::Process::memory) {
                obs.observe(i64::try_from(bytes).unwrap_or(i64::MAX), &[]);
            }
        })
        .build()
}

fn register_mem_virtual(
    meter: &Meter,
    system: Arc<Mutex<System>>,
    pid: Pid,
) -> ObservableUpDownCounter<i64> {
    meter
        .i64_observable_up_down_counter(semconv::PROCESS_MEMORY_VIRTUAL)
        .with_unit("By")
        .with_description("Virtual memory size committed by this process in bytes")
        .with_callback(move |obs| {
            if let Some(bytes) = with_process(&system, pid, sysinfo::Process::virtual_memory) {
                obs.observe(i64::try_from(bytes).unwrap_or(i64::MAX), &[]);
            }
        })
        .build()
}

fn register_disk_io(meter: &Meter, system: Arc<Mutex<System>>, pid: Pid) -> ObservableCounter<u64> {
    meter
        .u64_observable_counter(semconv::PROCESS_DISK_IO)
        .with_unit("By")
        .with_description(
            "Total bytes of disk I/O performed by this process since start, \
             split by direction (read / write). \
             On Windows this counts all I/O, not exclusively disk I/O.",
        )
        .with_callback(move |obs| {
            if let Some(usage) = with_process(&system, pid, sysinfo::Process::disk_usage) {
                obs.observe(
                    usage.total_read_bytes,
                    &[KeyValue::new(attribute::DISK_IO_DIRECTION, "read")],
                );
                obs.observe(
                    usage.total_written_bytes,
                    &[KeyValue::new(attribute::DISK_IO_DIRECTION, "write")],
                );
            }
        })
        .build()
}

fn register_thread_count(
    meter: &Meter,
    system: Arc<Mutex<System>>,
    pid: Pid,
) -> ObservableUpDownCounter<i64> {
    meter
        .i64_observable_up_down_counter(semconv::PROCESS_THREAD_COUNT)
        .with_unit("{thread}")
        .with_description(
            "Number of threads in this process. \
             On Linux this reflects the real task count via sysinfo; \
             on other platforms sysinfo returns 1.",
        )
        .with_callback(move |obs| {
            let count = with_process(&system, pid, |p| {
                p.tasks()
                    .and_then(|tasks| i64::try_from(tasks.len()).ok())
                    .unwrap_or(1_i64)
            })
            .unwrap_or(1_i64);
            obs.observe(count, &[]);
        })
        .build()
}

fn register_open_fds(
    meter: &Meter,
    system: Arc<Mutex<System>>,
    pid: Pid,
) -> ObservableUpDownCounter<i64> {
    meter
        .i64_observable_up_down_counter(semconv::PROCESS_OPEN_FILE_DESCRIPTOR_COUNT)
        .with_unit("{count}")
        .with_description(
            "Number of open file descriptors (Unix) or handles (Windows) \
             for this process, as reported by sysinfo.",
        )
        .with_callback(move |obs| {
            // open_files() returns None when the platform does not support it.
            if let Some(n) = with_process(&system, pid, sysinfo::Process::open_files).flatten() {
                obs.observe(i64::try_from(n).unwrap_or(i64::MAX), &[]);
            }
        })
        .build()
}

fn register_uptime(meter: &Meter, system: Arc<Mutex<System>>, pid: Pid) -> ObservableGauge<f64> {
    meter
        .f64_observable_gauge(semconv::PROCESS_UPTIME)
        .with_unit("s")
        .with_description("Time elapsed since this process started, in seconds")
        .with_callback(move |obs| {
            if let Some(secs) = with_process(&system, pid, sysinfo::Process::run_time) {
                // run_time is u64 seconds. f64 is lossless up to ~285 million years.
                #[allow(clippy::as_conversions, clippy::cast_precision_loss)]
                let uptime = secs as f64;
                obs.observe(uptime, &[]);
            }
        })
        .build()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use opentelemetry::metrics::MeterProvider as _;
    use opentelemetry_sdk::metrics::{
        InMemoryMetricExporter, SdkMeterProvider,
        data::{AggregatedMetrics, MetricData},
    };

    use super::*;

    fn test_provider() -> (SdkMeterProvider, InMemoryMetricExporter) {
        let exporter = InMemoryMetricExporter::default();
        let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter.clone()).build();
        let provider = SdkMeterProvider::builder().with_reader(reader).build();
        (provider, exporter)
    }

    fn find_metric_names(
        metrics: &[opentelemetry_sdk::metrics::data::ResourceMetrics],
    ) -> Vec<String> {
        metrics
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .map(|m| m.name().to_owned())
            .collect()
    }

    #[test]
    fn process_metrics_are_registered_and_flushed() {
        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");
        let _handles = ProcessMetricHandles::register(&meter);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let names = find_metric_names(&metrics);

        for expected in [
            semconv::PROCESS_CPU_TIME,
            semconv::PROCESS_CPU_UTILIZATION,
            semconv::PROCESS_MEMORY_USAGE,
            semconv::PROCESS_MEMORY_VIRTUAL,
            semconv::PROCESS_DISK_IO,
            semconv::PROCESS_THREAD_COUNT,
            semconv::PROCESS_UPTIME,
        ] {
            assert!(
                names.contains(&expected.to_owned()),
                "metric '{expected}' not found in: {names:?}",
            );
        }

        provider.shutdown().unwrap();
    }

    #[test]
    fn memory_usage_is_positive() {
        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");
        let _handles = ProcessMetricHandles::register(&meter);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let rss = metrics
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .find(|m| m.name() == semconv::PROCESS_MEMORY_USAGE)
            .expect("process.memory.usage not found");

        let value = match rss.data() {
            AggregatedMetrics::I64(MetricData::Sum(sum)) => sum
                .data_points()
                .map(opentelemetry_sdk::metrics::data::SumDataPoint::value)
                .sum::<i64>(),
            other => panic!("unexpected metric type: {other:?}"),
        };
        assert!(value > 0, "RSS should be positive, got {value}");

        provider.shutdown().unwrap();
    }

    #[test]
    fn disk_io_has_read_and_write_data_points() {
        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");
        let _handles = ProcessMetricHandles::register(&meter);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let disk = metrics
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .find(|m| m.name() == semconv::PROCESS_DISK_IO)
            .expect("process.disk.io not found");

        let directions: Vec<String> = match disk.data() {
            AggregatedMetrics::U64(MetricData::Sum(sum)) => sum
                .data_points()
                .filter_map(|dp| {
                    // attributes() returns an iterator directly — no .iter() needed
                    dp.attributes()
                        .find(|kv| kv.key.as_str() == attribute::DISK_IO_DIRECTION)
                        .map(|kv| kv.value.as_str().into_owned())
                })
                .collect(),
            other => panic!("unexpected metric type: {other:?}"),
        };

        assert!(
            directions.contains(&String::from("read")),
            "missing read data point, got: {directions:?}",
        );
        assert!(
            directions.contains(&String::from("write")),
            "missing write data point, got: {directions:?}",
        );

        provider.shutdown().unwrap();
    }

    #[test]
    fn uptime_is_positive() {
        let (provider, exporter) = test_provider();
        let meter = provider.meter("test");
        let _handles = ProcessMetricHandles::register(&meter);

        provider.force_flush().expect("flush failed");

        let metrics = exporter.get_finished_metrics().expect("no data");
        let uptime = metrics
            .iter()
            .flat_map(opentelemetry_sdk::metrics::data::ResourceMetrics::scope_metrics)
            .flat_map(opentelemetry_sdk::metrics::data::ScopeMetrics::metrics)
            .find(|m| m.name() == semconv::PROCESS_UPTIME)
            .expect("process.uptime not found");

        let value = match uptime.data() {
            // GaugeDataPoint::value() returns f64 directly (not &f64)
            AggregatedMetrics::F64(MetricData::Gauge(g)) => g
                .data_points()
                .map(opentelemetry_sdk::metrics::data::GaugeDataPoint::value)
                .next()
                .expect("no data points"),
            other => panic!("unexpected metric type: {other:?}"),
        };
        assert!(value >= 0.0, "uptime should be non-negative, got {value}");

        provider.shutdown().unwrap();
    }
}
