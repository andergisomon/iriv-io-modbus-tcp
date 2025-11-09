### Build

```bash
cargo build --target thumbv8m.main-none-eabi --release
```

Copy and rename ELF

```bash
cp ./target/thumbv8m.main-none-eabi/release/iriv-io-modbus-tcp firmware.elf
```

Flash using picotool

```bash
picotool load firmware.elf
```

Reboot

```bash
picotool reboot
```
