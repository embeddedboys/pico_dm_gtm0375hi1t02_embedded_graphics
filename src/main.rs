//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use cortex_m::asm;
use defmt::*;
use defmt_rtt as _;
use fugit::RateExtU32;
// use cortex_m::singleton;
use hal::{
    clocks::{ClocksManager, InitError},
    // dma::{double_buffer, single_buffer, DMAExt},
    gpio::{FunctionPio0, Pin},
    pac,
    pac::vreg_and_chip_reset::vreg::VSEL_A,
    pio::{Buffers, PIOExt, ShiftDirection},
    pll::{common_configs::PLL_USB_48MHZ, setup_pll_blocking},
    sio::Sio,
    vreg::set_voltage,
    // watchdog::Watchdog,
    xosc::setup_xosc_blocking,
    Clock,
    i2c::I2C,
};
use panic_probe as _;
use rp2040_hal as hal;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000; // Typically found in BSP crates
use rp_pico as bsp;

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::{Rgb888, Rgb565},
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};
use lib::{overclock, Pio8BitBus, ILI9488};
use overclock::overclock_configs::PLL_SYS_240MHZ;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    // let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    set_voltage(&mut pac.VREG_AND_CHIP_RESET, VSEL_A::VOLTAGE1_10);

    let xosc = setup_xosc_blocking(pac.XOSC, XOSC_CRYSTAL_FREQ.Hz())
        .map_err(InitError::XoscErr)
        .ok()
        .unwrap();
    let mut clocks = ClocksManager::new(pac.CLOCKS);

    let pll_sys = setup_pll_blocking(
        pac.PLL_SYS,
        xosc.operating_frequency().into(),
        PLL_SYS_240MHZ,
        &mut clocks,
        &mut pac.RESETS,
    )
    .map_err(InitError::PllError)
    .unwrap();
    let pll_usb = setup_pll_blocking(
        pac.PLL_USB,
        xosc.operating_frequency().into(),
        PLL_USB_48MHZ,
        &mut clocks,
        &mut pac.RESETS,
    )
    .map_err(InitError::PllError)
    .unwrap();

    clocks
        .init_default(&xosc, &pll_sys, &pll_usb)
        .map_err(InitError::ClockError)
        .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let program = pio_proc::pio_asm!(
        ".side_set 1"
        ".wrap_target",
        "   out pins, 16    side 0",
        "   nop             side 1",
        ".wrap"
    );

    let wr: Pin<_, FunctionPio0, _> = pins.gpio19.into_function();
    let wr_pin_id = wr.id().num;

    let dc = pins.gpio20.into_push_pull_output();
    let rst = pins.gpio18.into_push_pull_output();
    let bl = pins.gpio28.into_push_pull_output();

    let lcd_d0: Pin<_, FunctionPio0, _> = pins.gpio0.into_function();
    let lcd_d1: Pin<_, FunctionPio0, _> = pins.gpio1.into_function();
    let lcd_d2: Pin<_, FunctionPio0, _> = pins.gpio2.into_function();
    let lcd_d3: Pin<_, FunctionPio0, _> = pins.gpio3.into_function();
    let lcd_d4: Pin<_, FunctionPio0, _> = pins.gpio4.into_function();
    let lcd_d5: Pin<_, FunctionPio0, _> = pins.gpio5.into_function();
    let lcd_d6: Pin<_, FunctionPio0, _> = pins.gpio6.into_function();
    let lcd_d7: Pin<_, FunctionPio0, _> = pins.gpio7.into_function();

    let lcd_d0_pin_id = lcd_d0.id().num;

    let pindirs = [
        (wr_pin_id, hal::pio::PinDir::Output),
        (lcd_d0.id().num, hal::pio::PinDir::Output),
        (lcd_d1.id().num, hal::pio::PinDir::Output),
        (lcd_d2.id().num, hal::pio::PinDir::Output),
        (lcd_d3.id().num, hal::pio::PinDir::Output),
        (lcd_d4.id().num, hal::pio::PinDir::Output),
        (lcd_d5.id().num, hal::pio::PinDir::Output),
        (lcd_d6.id().num, hal::pio::PinDir::Output),
        (lcd_d7.id().num, hal::pio::PinDir::Output),
    ];

    let (mut pio, sm0, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
    let installed = pio.install(&program.program).unwrap();
    let (int, frac) = (1, 0); // as slow as possible (0 is interpreted as 65536)
    let (mut sm, _, tx) = rp2040_hal::pio::PIOBuilder::from_installed_program(installed)
        .side_set_pin_base(wr_pin_id)
        .out_pins(lcd_d0_pin_id, 8)
        .buffers(Buffers::OnlyTx)
        .clock_divisor_fixed_point(int, frac)
        .out_shift_direction(ShiftDirection::Right)
        .autopull(true)
        .pull_threshold(8)
        .build(sm0);
    sm.set_pindirs(pindirs);
    sm.start();

    info!("PIO block setuped");

    let di = Pio8BitBus::new(tx, dc);
    let mut display = ILI9488::new(di, Some(rst), Some(bl), 480, 320);
    display.init(&mut delay).unwrap();

    // Create styles used by the drawing operations.
    let light_blue = Rgb888::new(0x00, 0xd2, 0xff);
    let dark_blue = Rgb888::new(0x00, 0x14, 0x28);
    let thin_stroke = PrimitiveStyle::with_stroke(light_blue, 2);
    let thick_stroke = PrimitiveStyle::with_stroke(light_blue, 3);
    let border_stroke = PrimitiveStyleBuilder::new()
        .stroke_color(light_blue)
        .stroke_width(5)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();
    let fill = PrimitiveStyle::with_fill(light_blue);
    let character_style = MonoTextStyle::new(&FONT_10X20, light_blue);

    let yoffset = 14;

    for _ in 0..1 {
        display.color_converted().clear(dark_blue).unwrap();
    }

    // Draw a 3px wide outline around the display.
    display
        .bounding_box()
        .into_styled(border_stroke)
        .draw(&mut display.color_converted())
        .unwrap();

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(&mut display.color_converted())
    .unwrap();

    // Draw a filled square
    Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
        .into_styled(fill)
        .draw(&mut display.color_converted())
        .unwrap();

    // Draw a circle with a 3px wide stroke.
    Circle::new(Point::new(88, yoffset), 17)
        .into_styled(thick_stroke)
        .draw(&mut display.color_converted())
        .unwrap();

    // Draw centered text.
    let text = "embedded-graphics";
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(&mut display.color_converted())
    .unwrap();

    // let text = "touch anywhere to test";
    // Text::with_alignment(
    //     text,
    //     display.bounding_box().center() + Point::new(0, 30),
    //     character_style,
    //     Alignment::Center,
    // )
    // .draw(&mut display.color_converted())
    // .unwrap();

    let i2c = I2C::i2c1(
        pac.I2C1,
        pins.gpio26.reconfigure(),
        pins.gpio27.reconfigure(),
        400.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let irq_pin = pins.gpio21.into_pull_up_input();
    let mut touch = tsc2007::TSC2007::new(irq_pin, i2c).unwrap();
    let _ = touch.init(&mut delay);

    loop {
        asm::wfi();
        // touch.read().map_err(|_| ()).unwrap().map(|point| {
        //     // info!("x : {}, y : {}", point.0, point.1);
        //     Circle::with_center(Point::new((point.0) as _, (point.1) as _), 0)
        //     .into_styled(thick_stroke)
        //     .draw(&mut display.color_converted())
        //     .unwrap();
        // });
    }
}

mod tsc2007 {
    use cortex_m::delay::Delay;
    use defmt::info;
    use embedded_hal::digital::{InputPin, OutputPin};
    use embedded_hal::i2c::I2c;

    const TSC2007_DEF_ADDR: u8   = 0x48;
    const TSC2007_CMD_READ_X: u8 = 0xC0;
    const TSC2007_CMD_READ_Y: u8 = 0xD0;

    pub struct TSC2007<IRQ: InputPin, I2C: I2c> {
        irq: IRQ,
        i2c: I2C,
        addr: u8,
    }

    impl<PinE, IRQ: InputPin<Error = PinE>, I2C: I2c>
        TSC2007<IRQ, I2C>
    {
        pub fn new(irq: IRQ, i2c: I2C) -> Result<Self, PinE> {
            Ok(Self {
                irq,
                i2c,
                addr: TSC2007_DEF_ADDR,
            })
        }

        pub fn init(&mut self, delay_source: &mut Delay) -> Result<(), Error<PinE, I2C::Error>> {
            Ok(())
        }

        pub fn read_reg(&mut self, reg: u8) -> Result<u8, I2C::Error> {
            let mut readbuf: [u8; 1] = [0];
            self.i2c.write_read(self.addr, &[reg], &mut readbuf)?;
            Ok(readbuf[0])
        }

        pub fn read_reg_16(&mut self, reg: u8) -> Result<u16, I2C::Error> {
            let mut readbuf: [u8; 2] = [0; 2];
            self.i2c.write_read(self.addr, &[reg], &mut readbuf)?;
            Ok((readbuf[0] as u16) << 8 | (readbuf[1] as u16))
        }

        // pub fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), I2C::Error> {
        //     Ok(())
        // }

        pub fn is_pressed(&mut self) -> Result<bool, PinE> {
            self.irq.is_low()
        }

        pub fn read_x(&mut self) -> Result<u16, I2C::Error> {
            Ok(self.read_reg_16(TSC2007_CMD_READ_X)?)
        }

        pub fn read_y(&mut self) -> Result<u16, I2C::Error> {
            Ok(320 - (self.read_reg_16(TSC2007_CMD_READ_Y)? & 0x1fff))
        }

        pub fn read(&mut self) -> Result<Option<(u16, u16)>, Error<PinE, I2C::Error>> {
            match self.is_pressed() {
                Ok(pressed) => {
                    if !pressed {
                        Ok(None)
                    } else {
                        Ok(Some((
                            self.read_x().unwrap(),
                            self.read_y().unwrap(),
                        )))
                    }
                }
                Err(e) => {
                    Ok(None)
                }
            }
        }
    }

    pub enum Error<PinE, TransferE> {
        Pin(PinE),
        I2C(TransferE),
    }
}


// End of file
