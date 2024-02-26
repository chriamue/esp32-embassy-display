# esp32-embassy-display

## Run

```bash
. $HOME/export-esp.sh
cargo run --release
```


## Flashing

```bash
CRATE_CC_NO_DEFAULTS=1 cargo espflash flash --features=esp32s3 --release --monitor
cargo espflash flash --release --monitor
```

```bash
. $HOME/export-esp.sh
CRATE_CC_NO_DEFAULTS=1 cargo +esp run --no-default-features --features=esp32s3 --release --target xtensa-esp32s3-espidf
```
