# rustReverseShell  
  
First ever Rust project aimed to learn Rust basics. PoC of a Rust reverse shell, not expected to be used in real conditions.  
- Windows / Unix support
- Unsecure communications
- No AV bypass || anti reverse solution
- Only one shell process opened. Commands are processed by remote STDIN. STDOUT, STDERR are sent back.
  
```rust 
cargo run --bin server --release
cargo run --bin client --release
```