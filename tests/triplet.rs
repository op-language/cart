use cart::triplet::{TargetTriplet, TripletError};

#[test]
fn parse_three_part_triplet() {
    let t = TargetTriplet::parse("rp2A03-nintendo-nes").expect("parse");
    assert_eq!(t.cpu, "rp2A03");
    assert_eq!(t.manufacturer, "nintendo");
    assert_eq!(t.machine, "nes");
    assert_eq!(t.variant, "");
}

#[test]
fn parse_four_part_triplet() {
    let t = TargetTriplet::parse("rp2A03-nintendo-nes-ntsc").expect("parse");
    assert_eq!(t.cpu, "rp2A03");
    assert_eq!(t.manufacturer, "nintendo");
    assert_eq!(t.machine, "nes");
    assert_eq!(t.variant, "ntsc");
}

#[test]
fn parse_malformed_triplet() {
    let err = TargetTriplet::parse("bad").unwrap_err();
    assert!(matches!(err, TripletError::Malformed(_)));
}

#[test]
fn parse_two_part_triplet_fails() {
    let err = TargetTriplet::parse("cpu-vendor").unwrap_err();
    assert!(matches!(err, TripletError::Malformed(_)));
}

#[test]
fn as_str_roundtrip() {
    let t = TargetTriplet::parse("z80-sega-genesis").expect("parse");
    assert_eq!(t.as_str(), "z80-sega-genesis");
}

#[test]
fn as_str_with_variant() {
    let t = TargetTriplet::parse("mos6502-atari-800-ntsc").expect("parse");
    assert_eq!(t.as_str(), "mos6502-atari-800-ntsc");
}

#[test]
fn display_trait() {
    let t = TargetTriplet::parse("rp2A03-nintendo-nes-ntsc").expect("parse");
    assert_eq!(format!("{t}"), "rp2A03-nintendo-nes-ntsc");
}
