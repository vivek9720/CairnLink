# CairnLink

CairnLink is an offline Rust decoder for a fictional but realistic disaster-shelter
coordination exchange format. Emergency operations centers often exchange shelter
capacity, route access, stock ledgers, fragmentary radio packets, and field-report
templates while internet service is unreliable. CairnLink models that workflow as a
multi-stage binary/text bundle decoder with stateful reconciliation.

The crate is intentionally dependency-free and includes cargo-fuzz harnesses under
`fuzz/`. ClusterFuzzLite builds are handled by `.clusterfuzzlite/build.sh`; the build
uses only checked-in sources and a local `libfuzzer-sys` shim.

## Entry Points

- `decode_stream`: framed stream and nested bundle decoder.
- `decode_bundle`: shelter exchange container parser.
- `parse_frames`: raw frame parser.
- `run_script`: small automation bytecode interpreter.
- `replay_journal`: stock and checkpoint journal replayer.

The seed corpus contains valid near-miss exchanges for each harness.
