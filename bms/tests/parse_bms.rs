use bms::HeaderKey;
use std::path::Path;

#[test]
fn parse_test_bms() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest).join("tests").join("test.bms");
    let text = std::fs::read_to_string(&path).expect("failed to read test.bms");

    let b = bms::parse_bms(&text).expect("parse error");

    // headers (use helper APIs)
    assert_eq!(b.header(&HeaderKey::Title), Some("skin test"));
    assert_eq!(b.header_by_str("BPM"), Some("120"));
    assert_eq!(b.wav(1), Some("1_000.ogg"));

    // measure 1, channel 11: one non-empty at slot 1 with value "01"
    let m1 = b.measures.get(&1).expect("measure 1 missing");
    let ch11 = m1.iter().find(|c| c.channel == 11).expect("ch11 missing");
    let nonempty_count = ch11.data.iter().filter(|v| v.is_some()).count();
    assert_eq!(nonempty_count, 1);
    assert_eq!(ch11.data.get(1).and_then(|o| o.as_deref()), Some("01"));

    // measure 2, channel 58: expect at least two non-empty events
    let m2 = b.measures.get(&2).expect("measure 2 missing");
    let ch58 = m2.iter().find(|c| c.channel == 58).expect("ch58 missing");
    let c58_nonempty = ch58.data.iter().filter(|v| v.is_some()).count();
    assert!(c58_nonempty >= 2);

    // measure 3, channel 19: event at slot 0 = "01"
    let m3 = b.measures.get(&3).expect("measure 3 missing");
    let ch19 = m3.iter().find(|c| c.channel == 19).expect("ch19 missing");
    assert_eq!(ch19.data.get(0).and_then(|o| o.as_deref()), Some("01"));
}
