# Rust APK diff

This is an opt-in Rust port of the repository's `apkdiff.py` helper. It compares uncompressed
archive entries and ignores the same three APK signing metadata files. The Python script remains
available and unchanged for existing workflows.

Build and run it from the repository root:

```sh
cargo run --manifest-path Tools/apkdiff-rs/Cargo.toml -- path/to/first.apk path/to/second.apk
```

The command exits with status 0 when the APK contents match, 1 when they differ or the usage is
invalid, and 2 when an archive cannot be read. It is intentionally not part of the Android Gradle
build.

This small utility is a first, isolated Rust migration slice. Android UI and lifecycle code still
belongs to the existing Java/Kotlin app, and the Python plugin runtime, SDK, plugin contracts, and
build tooling remain intact.
