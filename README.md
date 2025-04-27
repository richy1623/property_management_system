# Property Management System

## How to run tests with coverage

### Locally

```shell
cargo tarpaulin --no-fail-fast --target-dir "target/tarpaulin" --exclude-files "target/*" --skip-clean --out html
```
