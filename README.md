### embedded_graphics porting for Pico_DM_QD3503728

## Hardware Requirements

- Raspberry Pi Pico board
- Pico_DM_QD3503728 display extends board
- A Micro-USB or USB-C cable
- (A CMSIS-DAP compatible SWD debugger)

## Software Requirements

1. Rust is needed before build this project, on linux or WSL we could install it easily by this:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2. Install toolchain for rp2040:
    ```bash
    rustup self update
    rustup update stable
    rustup target add thumbv6m-none-eabi
    ```

3. Install these helpful tools for debugging and flashing:
    ```bash
    # Useful to creating UF2 images for the RP2040 USB Bootloader
    cargo install elf2uf2-rs --locked
    # Useful for flashing over the SWD pins using a supported JTAG probe
    cargo install --locked probe-rs-tools
    ```

## Run on Pico

After all the software requirements are installed, we can build the project:
```bash
cargo build -r
```

### Depoly the firmware

1. Flash via CMSIS-DAP debugger:

    copy `50-cmsis-dap.rules` to `/etc/udev/rules.d/`
    ```bash
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    ```

    then flash the firmware
    ```bash
    probe-rs run --chip RP2040 --protocol swd target/thumbv6m-none-eabi/release/rp2040-project-template
    ```

2. Flash via RP2040 bootloader:

    Hold the `BOOTSEL` button and on the raspberry pi pico connect the usb cable to the board.
    ```bash
    elf2uf2-rs -d target/thumbv6m-none-eabi/release/rp2040-project-template
    ```

Or you can simply flash the firmware by the following steps:

1. modify the `runner` in `.cargo/config.toml` to suit your needs:
    ```toml
    runner = "probe-rs run --chip RP2040 --protocol swd"
    # runner = "elf2uf2-rs -d"
    ```
2. run the target
    ```toml
    cargo run -r
    ```