# VBAN API implemented in Rust

## API Examples

### Emitter
```rust
use rusty_vban::emitter::{EmitterBuilder, EmitterOptions};

EmitterBuilder::default()
    .ip_address("192.168.0.1")
    .stream_name("Mic")
    .port(6890) // Optional, default: 6890
    .channels(2) // Optional, default: 2
    .device("default") // Optional, default: "default"
    .backend("default") // Optional, default: "default"
    .build()
    .unwrap()
    .run(EmitterOptions::default())
    .unwrap();
```

### Receptor
```rust
use rusty_vban::receptor::{ReceptorBuilder, ReceptorOptions};

ReceptorBuilder::default()
    .latency(16) // Optional, default: 16
    .ip_address("192.168.0.1")
    .stream_name("Mic")
    .port(6890) // Optional, default: 6890
    .channels(2) // Optional, default: 2
    .device("default") // Optional, default: "default"
    .backend("default") // Optional, default: "default"
    .build()
    .unwrap()
    .run(ReceptorOptions::default())
    .unwrap();
```
