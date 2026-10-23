//! One-item lookahead for traces.
//!
//! [`Peekable`] wraps a trace and lets you inspect, or modify, the next item
//! before it is consumed. Every trace trait in this crate has a matching
//! extension trait (e.g. [`DelayTracePeekExt`]) that adds a `peekable()` method
//! to all of its implementors, and boxed trait objects (e.g.
//! `Box<dyn DelayTrace>`) get the same method without any import.
//!
//! The wrapper implements the trace trait of the trace it wraps, so items are
//! consumed with the usual method (e.g. `next_delay()`) and a [`Peekable`] can
//! be used anywhere the original trace was.
//!
//! ## Examples
//!
//! ```
//! # use netem_trace::model::StaticDelayConfig;
//! # use netem_trace::{Delay, DelayTrace, Duration};
//! use netem_trace::peek::DelayTracePeekExt;
//!
//! let mut trace = StaticDelayConfig::new()
//!     .delay(Delay::from_millis(10))
//!     .duration(Duration::from_secs(1))
//!     .build()
//!     .peekable();
//! let item = (Delay::from_millis(10), Duration::from_secs(1));
//! assert_eq!(trace.peek(), Some(&item));
//! assert_eq!(trace.next_delay(), Some(item));
//! assert_eq!(trace.peek(), None);
//! ```
//!
//! A boxed trace, such as one built from a configuration, works the same way:
//!
//! ```
//! # use netem_trace::model::{DelayTraceConfig, StaticDelayConfig};
//! # use netem_trace::{Delay, DelayTrace, Duration};
//! let config = Box::new(
//!     StaticDelayConfig::new()
//!         .delay(Delay::from_millis(10))
//!         .duration(Duration::from_secs(1)),
//! ) as Box<dyn DelayTraceConfig>;
//! let mut trace = config.into_model().peekable();
//! let item = (Delay::from_millis(10), Duration::from_secs(1));
//! assert_eq!(trace.peek(), Some(&item));
//! assert_eq!(trace.next_delay(), Some(item));
//! ```
use super::{Bandwidth, Delay, DuplicatePattern, Duration, LossPattern, RwndDecision};
use super::{BwTrace, DelayPerPacketTrace, DelayTrace, DuplicateTrace, LossTrace, RwndTrace};

/// A trace wrapper that can look ahead one item.
///
/// `Trace` is the wrapped trace and `Item` is what it yields. It is usually
/// created with a `peekable()` method rather than with [`Peekable::new`]; see
/// the [module documentation](self) for examples.
///
/// An item is pulled from the wrapped trace only when it is peeked or consumed,
/// and a peeked item is held until it is consumed.
pub struct Peekable<Trace, Item> {
    inner: Trace,
    next: fn(&mut Trace) -> Option<Item>,
    peeked: Option<Item>,
}

impl<Trace, Item> Peekable<Trace, Item> {
    /// Wrap `inner`, using `next` to pull its items.
    pub fn new(inner: Trace, next: fn(&mut Trace) -> Option<Item>) -> Self {
        Self {
            inner,
            next,
            peeked: None,
        }
    }

    #[inline(always)]
    fn update(&mut self) {
        if self.peeked.is_none() {
            self.peeked = (self.next)(&mut self.inner);
        }
    }

    /// Return a reference to the next item without consuming it, or `None` if
    /// the trace has ended.
    pub fn peek(&mut self) -> Option<&Item> {
        self.update();
        self.peeked.as_ref()
    }

    /// Return a mutable reference to the next item without consuming it, or
    /// `None` if the trace has ended. Changes are seen by the next consumer.
    pub fn peek_mut(&mut self) -> Option<&mut Item> {
        self.update();
        self.peeked.as_mut()
    }

    #[inline(always)]
    fn take_next(&mut self) -> Option<Item> {
        self.update();
        self.peeked.take()
    }
}

macro_rules! impl_peekable_for {
    ($ext:ident, $trace:ident, $next:ident, $item:ty) => {
        #[doc = concat!("Adds `peekable()` to every type that implements [`", stringify!($trace), "`].")]
        ///
        #[doc = concat!("For `Box<dyn ", stringify!($trace), ">`, the same method is provided on the trait object itself.")]
        pub trait $ext: $trace + Sized {
            /// Wrap `self` in a [`Peekable`] that can look ahead one item.
            fn peekable(self) -> Peekable<Self, $item> {
                Peekable::new(self, <Self as $trace>::$next)
            }
        }

        impl<T: $trace> $ext for T {}

        // `Box<dyn $trace>` does not implement `$trace` itself, so the
        // blanket impl above does not cover it. An inherent method on the
        // trait object gives boxed traces the same `peekable()` call.
        impl dyn $trace {
            /// Wrap the boxed trace in a [`Peekable`] that can look ahead one item.
            pub fn peekable(self: Box<Self>) -> Peekable<Box<Self>, $item> {
                Peekable::new(self, |inner| inner.$next())
            }
        }

        // Make the wrapper a drop-in replacement for the wrapped trace.
        impl<T: $trace> $trace for Peekable<T, $item> {
            fn $next(&mut self) -> Option<$item> {
                self.take_next()
            }
        }

        impl $trace for Peekable<Box<dyn $trace>, $item> {
            fn $next(&mut self) -> Option<$item> {
                self.take_next()
            }
        }
    };
}

impl_peekable_for!(BwTracePeekExt, BwTrace, next_bw, (Bandwidth, Duration));
impl_peekable_for!(DelayTracePeekExt, DelayTrace, next_delay, (Delay, Duration));
impl_peekable_for!(
    DelayPerPacketTracePeekExt,
    DelayPerPacketTrace,
    next_delay,
    Delay
);
impl_peekable_for!(
    LossTracePeekExt,
    LossTrace,
    next_loss,
    (LossPattern, Duration)
);
impl_peekable_for!(
    DuplicateTracePeekExt,
    DuplicateTrace,
    next_duplicate,
    (DuplicatePattern, Duration)
);
impl_peekable_for!(
    RwndTracePeekExt,
    RwndTrace,
    next_rwnd,
    (RwndDecision, Duration)
);

#[cfg(test)]
mod test {
    use super::*;

    /// A trace that yields the given delays, each lasting one second.
    struct VecDelay(std::vec::IntoIter<Delay>);

    impl VecDelay {
        fn new(millis: &[u64]) -> Self {
            let delays: Vec<_> = millis.iter().map(|&ms| Delay::from_millis(ms)).collect();
            Self(delays.into_iter())
        }
    }

    impl DelayTrace for VecDelay {
        fn next_delay(&mut self) -> Option<(Delay, Duration)> {
            self.0.next().map(|delay| (delay, Duration::from_secs(1)))
        }
    }

    fn item(ms: u64) -> (Delay, Duration) {
        (Delay::from_millis(ms), Duration::from_secs(1))
    }

    // Accepts any trace, to check the wrapper is usable as a drop-in replacement.
    fn drain(trace: &mut impl DelayTrace) -> Vec<(Delay, Duration)> {
        std::iter::from_fn(|| trace.next_delay()).collect()
    }

    #[test]
    fn test_peek_does_not_consume() {
        let mut trace = VecDelay::new(&[10, 20]).peekable();
        assert_eq!(trace.peek(), Some(&item(10)));
        assert_eq!(trace.peek(), Some(&item(10)));
        assert_eq!(trace.next_delay(), Some(item(10)));
        assert_eq!(trace.peek(), Some(&item(20)));
        assert_eq!(trace.next_delay(), Some(item(20)));
        assert_eq!(trace.peek(), None);
        assert_eq!(trace.next_delay(), None);
    }

    #[test]
    fn test_peek_mut_modifies_next() {
        let mut trace = VecDelay::new(&[10, 20]).peekable();
        if let Some((delay, _)) = trace.peek_mut() {
            *delay = Delay::from_millis(15);
        }
        assert_eq!(trace.next_delay(), Some(item(15)));
        assert_eq!(trace.next_delay(), Some(item(20)));
    }

    #[test]
    fn test_peekable_is_drop_in_trace() {
        let mut trace = VecDelay::new(&[10, 20, 30]).peekable();
        assert_eq!(trace.peek(), Some(&item(10)));
        assert_eq!(drain(&mut trace), vec![item(10), item(20), item(30)]);
    }

    #[test]
    fn test_boxed_dyn_trace() {
        let boxed: Box<dyn DelayTrace> = Box::new(VecDelay::new(&[10, 20]));
        let mut trace = boxed.peekable();
        assert_eq!(trace.peek(), Some(&item(10)));
        assert_eq!(drain(&mut trace), vec![item(10), item(20)]);
    }

    #[test]
    fn test_boxed_peekable_as_dyn_trace() {
        let boxed: Box<dyn DelayTrace> = Box::new(VecDelay::new(&[10, 20]));
        let mut trace: Box<dyn DelayTrace> = Box::new(boxed.peekable());
        assert_eq!(trace.next_delay(), Some(item(10)));
        assert_eq!(trace.next_delay(), Some(item(20)));
        assert_eq!(trace.next_delay(), None);
    }

    #[cfg(feature = "delay-model")]
    mod delay_model {
        use super::*;
        use crate::model::{DelayTraceConfig, StaticDelayConfig};

        #[test]
        fn test_static_delay() {
            let mut trace = StaticDelayConfig::new()
                .delay(Delay::from_millis(10))
                .duration(Duration::from_secs(1))
                .build()
                .peekable();
            assert_eq!(trace.peek(), Some(&item(10)));
            assert_eq!(trace.next_delay(), Some(item(10)));
            assert_eq!(trace.peek(), None);
            assert_eq!(trace.next_delay(), None);
        }

        #[test]
        fn test_model_from_config() {
            let config = Box::new(
                StaticDelayConfig::new()
                    .delay(Delay::from_millis(10))
                    .duration(Duration::from_secs(1)),
            ) as Box<dyn DelayTraceConfig>;
            let mut trace = config.into_model().peekable();
            assert_eq!(trace.peek(), Some(&item(10)));
            assert_eq!(drain(&mut trace), vec![item(10)]);
        }
    }

    // `DelayPerPacketTrace` shares the `next_delay` name with `DelayTrace` but
    // yields a bare `Delay`, and `Box<dyn DelayPerPacketTrace>` has a `From`
    // impl for every model, so check that its boxed form resolves on its own.
    #[cfg(feature = "delay-per-packet-model")]
    mod delay_per_packet_model {
        use super::*;
        use crate::model::StaticDelayPerPacketConfig;

        #[test]
        fn test_boxed_static_delay_per_packet() {
            let boxed: Box<dyn DelayPerPacketTrace> = StaticDelayPerPacketConfig::new()
                .delay(Delay::from_millis(10))
                .count(2)
                .build()
                .into();
            let mut trace = boxed.peekable();
            assert_eq!(trace.peek(), Some(&Delay::from_millis(10)));
            assert_eq!(trace.next_delay(), Some(Delay::from_millis(10)));
            assert_eq!(trace.next_delay(), Some(Delay::from_millis(10)));
            assert_eq!(trace.peek(), None);
        }
    }
}
