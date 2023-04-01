# VBAN API implemented in Rust

## Examples

### Emitter
```rust
emitter::EmitterBuilder::default()
    .ip_address("192.168.0.1")
    .stream_name("Mic")
    .port(6890)
    .channels(2)
    .device("default")
    .build()?
    .start()?;
```

### Receptor
```rust
receptor::ReceptorBuilder::default()
    .latency(16)
    .ip_address("192.168.0.1")
    .stream_name("Mic")
    .port(6890)
    .channels(2)
    .device("default")
    .build()?
    .start()?;
```
