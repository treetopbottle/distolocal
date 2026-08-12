use distolocal::application::append::{AppendCommand, AppendHandler};
use distolocal::application::read_stream::{ReadStreamHandler, ReadStreamQuery};
use distolocal::domain::stream::{Event, StreamId};
use distolocal::infrastructure::storage::in_memory::InMemoryStreamStore;

// Given a new stream ID "orders-123" with no existing events
// When 2 events are appended with no expected version
// Then the stream contains 2 events at positions 0 and 1
#[test]
fn appending_to_a_new_stream_assigns_zero_based_positions() {
    let store = InMemoryStreamStore::new();
    let append = AppendHandler::new(&store);

    let positions = append.handle(AppendCommand {
        stream_id: StreamId::new("orders-123"),
        events: vec![
            Event::new(b"order-placed".to_vec()),
            Event::new(b"order-paid".to_vec()),
        ],
    });

    assert_eq!(positions, vec![0, 1]);
}

// Given a stream "orders-123" with 2 events already appended
// When ReadStream is called for "orders-123"
// Then the 2 events are returned in append order
#[test]
fn reading_a_stream_returns_events_in_append_order() {
    let store = InMemoryStreamStore::new();
    let append = AppendHandler::new(&store);
    let read = ReadStreamHandler::new(&store);
    let stream_id = StreamId::new("orders-123");

    let appended = vec![
        Event::new(b"order-placed".to_vec()),
        Event::new(b"order-paid".to_vec()),
    ];
    append.handle(AppendCommand {
        stream_id: stream_id.clone(),
        events: appended.clone(),
    });

    let events = read.handle(ReadStreamQuery { stream_id });

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].position, 0);
    assert_eq!(events[0].event, appended[0]);
    assert_eq!(events[1].position, 1);
    assert_eq!(events[1].event, appended[1]);
}
