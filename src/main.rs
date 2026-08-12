use distolocal::application::append::{AppendCommand, AppendHandler};
use distolocal::application::read_stream::{ReadStreamHandler, ReadStreamQuery};
use distolocal::domain::stream::{Event, StreamId};
use distolocal::infrastructure::storage::in_memory::InMemoryStreamStore;

mod observability;

fn main() {
    observability::init();

    let store = InMemoryStreamStore::new();
    let append = AppendHandler::new(&store);
    let read = ReadStreamHandler::new(&store);

    let stream_id = StreamId::new("example-stream");
    append.handle(AppendCommand {
        stream_id: stream_id.clone(),
        events: vec![Event::new(b"example-event".to_vec())],
    });

    for stored in read.handle(ReadStreamQuery { stream_id }) {
        println!("{} -> {:?}", stored.position, stored.event.payload);
    }
}
