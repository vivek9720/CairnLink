#[test]
fn parses_manifest_text() {
    let input = b"site=harbor-east\nincident=flood-plain\nmode=monitor\ncontact=ops\n";
    let state = cairnlink::decode_stream(input).expect("manifest text should parse");
    assert!(state.manifest.site_id.contains("harbor"));
}

#[test]
fn parses_empty_script() {
    let result = cairnlink::run_script(&[]).expect("empty script is valid");
    assert_eq!(result.accumulator, 0);
}
