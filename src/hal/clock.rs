use commkit::Instant;

/// A monotonic microsecond clock
///
/// Construction is left to each implementation, since timers are too hardware dependent to share a `new`
pub trait Clock {
    /// Time since the clock started
    fn now(&mut self) -> Instant;
}
