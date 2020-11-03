# SDLang parser for Rust

🚧 **Under construction** 🚧

## Status

- [x] Scanner
- [ ] Parser

declaration -> node
node -> IDENTIFIER attribute
attribute -> (IDENTIFIER EQUAL value)\*
literal -> STRING | NUMBER | DATE | FLOAT64 | FLOAT32 | DECIMAL128 | LONG | OFF | ON | TRUE | FALSE | NULL
