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

### 部署固件

1. 通过 CMSIS-DAP 调试器进行下载：

    拷贝 `50-cmsis-dap.rules` 至 `/etc/udev/rules.d/` 下
    ```bash
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    ```

    然后通过如下指令下载固件到板子
    ```bash
    probe-rs run --chip RP2040 --protocol swd target/thumbv6m-none-eabi/release/rp2040-project-template
    ```

2. 通过RP2040引导加载程序下载：

    按住 Raspberry Pi Pico 上的`BOOTSEL`按钮，将 USB 电缆连接到开发板。
    ```bash
    elf2uf2-rs -d target/thumbv6m-none-eabi/release/rp2040-project-template
    ```

或者您也可以按照以下步骤简单地下载固件：

1. 修改 `.cargo/config.toml` 中的 `runner` 以满足你的需求：
    ```toml
    runner = "probe-rs run --chip RP2040 --protocol swd"
    # runner = "elf2uf2-rs -d"
    ```
2. 运行目标程序
    ```toml
    cargo run -r
    ```