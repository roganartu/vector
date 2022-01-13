#![deny(missing_docs)]
//! This module contains the definitions and wrapper types for handling
//! arrays of type `Event`, in the various forms they may appear.

use std::iter;

use smallvec::{smallvec_inline, SmallVec};

use super::{Event, LogEvent, Metric};
use crate::ByteSizeOf;

/// The core trait to abstract over any type that may work as an array
/// of events. This is effectively the same as the standard
/// `IntoIterator<Item = Event>` implementations, but that would
/// conflict with the base implementation for the type aliases below.
pub trait EventContainer: ByteSizeOf {
    /// The type of `Iterator` used to turn this container into events.
    type IntoIter: Iterator<Item = Event>;
    /// Turn this container into an iterator of events.
    fn into_events(self) -> Self::IntoIter;
}

impl EventContainer for Event {
    type IntoIter = iter::Once<Event>;
    fn into_events(self) -> Self::IntoIter {
        iter::once(self)
    }
}

impl EventContainer for LogEvent {
    type IntoIter = iter::Once<Event>;
    fn into_events(self) -> Self::IntoIter {
        iter::once(self.into())
    }
}

impl EventContainer for Metric {
    type IntoIter = iter::Once<Event>;
    fn into_events(self) -> Self::IntoIter {
        iter::once(self.into())
    }
}

/// The type alias for an array of `LogEvent` elements.
pub type LogArray = SmallVec<[LogEvent; 1]>;

impl EventContainer for LogArray {
    type IntoIter = iter::Map<smallvec::IntoIter<[LogEvent; 1]>, fn(LogEvent) -> Event>;
    fn into_events(self) -> Self::IntoIter {
        self.into_iter().map(Into::into)
    }
}

/// The type alias for an array of `Metric` elements.
pub type MetricArray = SmallVec<[Metric; 1]>;

impl EventContainer for MetricArray {
    type IntoIter = iter::Map<smallvec::IntoIter<[Metric; 1]>, fn(Metric) -> Event>;
    fn into_events(self) -> Self::IntoIter {
        self.into_iter().map(Into::into)
    }
}

/// An array of one of the `Event` variants exclusively.
pub enum EventArray {
    /// An array of type `LogEvent`
    Logs(LogArray),
    /// An array of type `Metric`
    Metrics(MetricArray),
}

impl From<Event> for EventArray {
    fn from(event: Event) -> Self {
        match event {
            Event::Log(log) => Self::Logs(smallvec_inline![log]),
            Event::Metric(metric) => Self::Metrics(smallvec_inline![metric]),
        }
    }
}

impl ByteSizeOf for EventArray {
    fn allocated_bytes(&self) -> usize {
        match self {
            Self::Logs(a) => a.allocated_bytes(),
            Self::Metrics(a) => a.allocated_bytes(),
        }
    }
}

impl EventContainer for EventArray {
    type IntoIter = EventArrayIntoIter;

    fn into_events(self) -> Self::IntoIter {
        match self {
            Self::Logs(a) => EventArrayIntoIter::Logs(a.into_iter()),
            Self::Metrics(a) => EventArrayIntoIter::Metrics(a.into_iter()),
        }
    }
}

/// The iterator type for `EventArray`.
pub enum EventArrayIntoIter {
    /// An iterator over type `LogEvent`.
    Logs(smallvec::IntoIter<[LogEvent; 1]>),
    /// An iterator over type `Metric`.
    Metrics(smallvec::IntoIter<[Metric; 1]>),
}

impl Iterator for EventArrayIntoIter {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Logs(i) => i.next().map(Into::into),
            Self::Metrics(i) => i.next().map(Into::into),
        }
    }
}
