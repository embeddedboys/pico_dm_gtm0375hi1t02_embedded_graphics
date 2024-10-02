### Pico_DM_QD3503728 的 embedded_graphics移植

## TODO

- [x] 显示驱动 GPIO
- [ ] 显示驱动 PIO
- [ ] 显示驱动 PIO+DMA
- [ ] 支持触摸功能

## 硬件要求

- Raspberry Pi Pico 核心板
- Pico_DM_QD3503728 显示拓展板
- 一根 Micro-USB 或者 USB-C 线缆
- （与 CMSIS-DAP 兼容的 SWD 调试器）

## 软件要求

1. 在构建此项目之前需要 Rust，在 Linux 或 WSL 上我们可以通过以下方式轻松安装它：
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2. 为rp2040安装rust工具链：
    ```bash
    rustup self update
    rustup update stable
    rustup target add thumbv6m-none-eabi
    ```

3. 安装这些有助于调试和下载的工具:
    ```bash
    # Useful to creating UF2 images for the RP2040 USB Bootloader
    cargo install elf2uf2-rs --locked
    # Useful for flashing over the SWD pins using a supported JTAG probe
    cargo install --locked probe-rs-tools
    ```

## 在 Pico 上运行

安装完所有必需的软件后，我们可以构建项目：
```bash
cargo build -r
```

编译指定example
```bash
cargo build -r --example demo-text-tga
```

### 部署固件

有两种方式将编译好的文件烧录到Pico

1. 通过 CMSIS-DAP 调试器进行下载：

    你可能需要先配置udev rules才能让cmsis-dap得以识别到，复制工程目录下的`50-cmsis-dap.rules`，
    到`/etc/udev/rules.d/`路径下，然后执行
    ```bash
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    ```

    使用如下命令通过调试器烧录，并监视RTT调试信息
    ```bash
    probe-rs run --chip RP2040 --protocol swd target/thumbv6m-none-eabi/release/rp2040-project-template
    ```

2. 通过RP2040的bootloader UF2烧录

    按住核心板的`BOOTSEL`按键，插入USB线，或者在连接有线的情况下，按下拓展板上的复位键，让RP2040进入UF2下载模式，再通过如下命令将UF2文件下载至RP2040。
    ```bash
    elf2uf2-rs -d target/thumbv6m-none-eabi/release/rp2040-project-template
    ```

或者你可以简单地运行如下命令，编译并将文件烧录到RP2040。

1. 修改 `.cargo/config.toml` 中的 `runner` 以满足你的需求：
    ```toml
    runner = "probe-rs run --chip RP2040 --protocol swd"
    # runner = "elf2uf2-rs -d"
    ```
2. 运行目标程序
    ```toml
    cargo run -r
    ```