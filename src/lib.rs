//! Bridge between the `coz` Causal Profiler and tracing
//!
//! This crate takes advantage of places where tracing exposes tracepoints rust code, letting coz
//! see these tracepoints as latency for spans and throughput for events.
//!
//! For usage information, consult the [`README.md` for the `coz`
//! repository][coz-readme] as well as the [`README.md` for
//! `coz-rs`][rust-readme].
//!
//! [coz-readme]: https://github.com/plasma-umass/coz/blob/master/README.md
//! [rust-readme]: https://github.com/alexcrichton/coz-rs/blob/master/README.md

// Reexport this for the times its needed
pub use coz::thread_init;
use coz::Counter;

use std::sync::atomic::{Ordering, AtomicUsize};

use tracing::{
    Id, Metadata, Event,
    span,
    subscriber::{self, Subscriber},
};

use chashmap::CHashMap;

struct ThroughputCounter {
    start: Counter,
    end: Counter,
}

impl ThroughputCounter {
    fn new(name: &'static str) -> Self {
        ThroughputCounter {
            start: Counter::begin(name),
            end: Counter::end(name),
        }
    }
}

/// A bridge between the coz-profiler and tracing
pub struct TracingCozBridge {
    next_id: AtomicUsize,
    latency_counters: CHashMap<&'static str, Counter>,
    throughput_counters: CHashMap<Id, ThroughputCounter>,
    idents: CHashMap<&'static str, Id>,
}

impl TracingCozBridge {
    /// Creates a bridge between tracing and the coz profiler
    pub fn new() -> Self {
        TracingCozBridge {
            next_id: AtomicUsize::new(1),
            latency_counters: CHashMap::new(),
            throughput_counters: CHashMap::new(),
            idents: CHashMap::new(),
        }
    }

    fn next_id(&self) -> Id {
        Id::from_u64(self.next_id.fetch_add(1, Ordering::SeqCst) as u64)
    }
}

impl Subscriber for TracingCozBridge {
    fn register_callsite(&self, _meta: &Metadata<'_>) -> subscriber::Interest {
        subscriber::Interest::always()
    }

    fn new_span(&self, new_span: &span::Attributes<'_>) -> Id {
        let name = new_span.metadata().name();
        let throughput_counters = &self.throughput_counters;
        self.idents.upsert(name, || {
            let next_id = self.next_id();
            throughput_counters.upsert(next_id.clone(), || ThroughputCounter::new(name), |_| ());
            next_id
        }, |_| ());

        (*self.idents.get(name).unwrap()).clone()
    }

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {
        // ignored
    }

    fn record(&self, _: &Id, _values: &span::Record<'_>) {
        // ignored
    }

    fn event(&self, event: &Event<'_>) {
        let name = event.metadata().name();
        self.latency_counters.upsert(name, || Counter::progress(name), |cnt| cnt.increment());
    }

    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn enter(&self, span: &Id) {
        self.throughput_counters.get(span).unwrap().start.increment()
    }

    fn exit(&self, span: &Id) {
        self.throughput_counters.get(span).unwrap().end.increment()
    }
}
