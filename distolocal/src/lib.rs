use std::collections::HashMap;
use std::fmt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Where the Store gets the times it assigns.
pub trait Clock {
    fn now(&self) -> OffsetDateTime;
}

/// The clock a Node runs on in production: the machine's wall clock, in UTC.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// An RFC 3339 time, only built from an `OffsetDateTime`.
#[derive(Debug, Clone, PartialEq)]
pub struct FormattedDateTime(String);

impl From<OffsetDateTime> for FormattedDateTime {
    fn from(moment: OffsetDateTime) -> Self {
        FormattedDateTime(
            moment
                .format(&Rfc3339)
                .expect("formatting a time as RFC 3339 should never fail"),
        )
    }
}

impl fmt::Display for FormattedDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub event_id: String,
    pub stream_id: String,
    /// A count per Node that has appended to this Stream; a missing Node
    /// counts as 0.
    pub vector_clock: HashMap<String, u64>,
    pub event_type: String,
    /// When the Store appended this Event, read from its clock.
    pub created_at: FormattedDateTime,
    /// `data` and `metadata` are opaque bytes, stored as given.
    pub data: Vec<u8>,
    pub metadata: Vec<u8>,
}

/// An Event as the Application supplies it: everything but the `vector_clock`
/// and `created_at` the Store assigns.
#[derive(Debug)]
pub struct PendingEvent {
    pub event_id: String,
    pub event_type: String,
    pub data: Vec<u8>,
    pub metadata: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamStatus {
    Open,
    /// `closed_at` is when the Store closed the Stream, read from its clock.
    Closed {
        closed_at: FormattedDateTime,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub stream_id: String,
    pub status: StreamStatus,
    pub created_at: FormattedDateTime,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    StreamNotFound {
        stream_id: String,
    },
    StreamClosed {
        stream_id: String,
    },
    StreamNotClosed {
        stream_id: String,
    },
    /// The Event already stored under that `event_id`, boxed to keep
    /// `Result<_, Error>` small.
    EventIdConflict {
        stored: Box<Event>,
    },
}

pub struct Store {
    node_id: String,
    clock: Box<dyn Clock>,
    streams: HashMap<String, (Stream, Vec<Event>)>,
}

impl Store {
    pub fn new(node_id: &str, clock: impl Clock + 'static) -> Self {
        Store {
            node_id: node_id.to_string(),
            clock: Box::new(clock),
            streams: HashMap::new(),
        }
    }

    // Every lookup of an existing Stream goes through here or `stream_mut`.
    fn stream(&self, stream_id: &str) -> Result<&(Stream, Vec<Event>), Error> {
        self.streams
            .get(stream_id)
            .ok_or_else(|| Self::stream_not_found(stream_id))
    }

    // `stream`, for an operation that changes what it finds.
    fn stream_mut<'a>(
        streams: &'a mut HashMap<String, (Stream, Vec<Event>)>,
        stream_id: &str,
    ) -> Result<&'a mut (Stream, Vec<Event>), Error> {
        streams
            .get_mut(stream_id)
            .ok_or_else(|| Self::stream_not_found(stream_id))
    }

    fn stream_not_found(stream_id: &str) -> Error {
        Error::StreamNotFound {
            stream_id: stream_id.to_string(),
        }
    }

    // Every operation that a closed Stream refuses checks here first.
    fn ensure_open(stream: &Stream) -> Result<(), Error> {
        match stream.status {
            StreamStatus::Open => Ok(()),
            StreamStatus::Closed { .. } => Err(Error::StreamClosed {
                stream_id: stream.stream_id.clone(),
            }),
        }
    }

    /// Creates the Stream, or returns it unchanged if it already exists and
    /// is open. A closed Stream's id is not a way to reopen it.
    pub fn create_stream(&mut self, stream_id: &str) -> Result<Stream, Error> {
        if let Some((stream, _)) = self.streams.get(stream_id) {
            Self::ensure_open(stream)?;
            return Ok(stream.clone());
        }

        let stream = Stream {
            stream_id: stream_id.to_string(),
            status: StreamStatus::Open,
            created_at: self.clock.now().into(),
        };
        self.streams
            .insert(stream_id.to_string(), (stream.clone(), Vec::new()));
        Ok(stream)
    }

    /// The Stream's own record, without its Events.
    pub fn get_stream(&self, stream_id: &str) -> Result<&Stream, Error> {
        let (stream, _) = self.stream(stream_id)?;

        Ok(stream)
    }

    /// All the Events in a Stream, in append order. The Events stay owned by
    /// the Store; call `.to_vec()` for a copy.
    pub fn get_events(&self, stream_id: &str) -> Result<&[Event], Error> {
        let (_, events) = self.stream(stream_id)?;

        Ok(events)
    }

    /// Closes the Stream, or returns it unchanged if it is already closed.
    pub fn close_stream(&mut self, stream_id: &str) -> Result<Stream, Error> {
        let (stream, _) = Self::stream_mut(&mut self.streams, stream_id)?;

        if stream.status == StreamStatus::Open {
            stream.status = StreamStatus::Closed {
                closed_at: self.clock.now().into(),
            };
        }
        Ok(stream.clone())
    }

    /// Appends an Event to an existing, open Stream, marking it with this
    /// Node's next count for that Stream. An `event_id` the Stream already
    /// holds is a conflict, whatever the rest of the Event says.
    pub fn append_event(&mut self, stream_id: &str, event: PendingEvent) -> Result<Event, Error> {
        let (stream, events) = Self::stream_mut(&mut self.streams, stream_id)?;
        Self::ensure_open(stream)?;

        if let Some(stored) = events
            .iter()
            .find(|stored| stored.event_id == event.event_id)
        {
            return Err(Error::EventIdConflict {
                stored: Box::new(stored.clone()),
            });
        }

        let mut vector_clock = events
            .last()
            .map(|last| last.vector_clock.clone())
            .unwrap_or_default();
        *vector_clock.entry(self.node_id.clone()).or_insert(0) += 1;

        let appended = Event {
            event_id: event.event_id,
            stream_id: stream_id.to_string(),
            vector_clock,
            event_type: event.event_type,
            created_at: self.clock.now().into(),
            data: event.data,
            metadata: event.metadata,
        };
        events.push(appended.clone());
        Ok(appended)
    }

    /// Test-only: every Stream id, in no particular order.
    #[doc(hidden)]
    pub fn stream_ids(&self) -> Vec<&str> {
        self.streams.keys().map(String::as_str).collect()
    }
}
