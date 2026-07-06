#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Ok(frames) = cairnlink::parse_frames(data) { let mut state = cairnlink::SessionState::new(); for frame in frames { let _ = state.apply_frame(frame); } let _ = state.finalize(); }
});
