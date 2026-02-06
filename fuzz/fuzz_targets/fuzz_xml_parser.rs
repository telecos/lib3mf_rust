#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz XML parsing with quick-xml
    // This tests the underlying XML parser's robustness with malformed XML
    use quick_xml::Reader;
    use quick_xml::events::Event;
    
    let mut reader = Reader::from_reader(data);
    reader.config_mut().trim_text(true);
    
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Err(_) => break, // Ignore errors, we're just testing for crashes
            _ => (),
        }
        buf.clear();
    }
});
