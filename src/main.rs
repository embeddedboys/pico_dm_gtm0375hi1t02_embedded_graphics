//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;
use defmt_rtt as _;
use panic_probe as _;
use rp_pico as bsp;

use mipidsi::{models::ILI9486Rgb565, options::Orientation, options::Rotation};
use mipidsi::options::ColorOrder;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{
        Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle,
    },
    text::{Alignment, Text},
};

use display_interface_parallel_gpio::{Generic16BitBus, PGPIO16BitInterface};
use mipidsi::Builder;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac,
    sio::Sio,
    watchdog::Watchdog,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = DelayCompat(cortex_m::delay::Delay::new(
        core.SYST,
        clocks.system_clock.freq().to_Hz(),
    ));

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let rst = pins.gpio22.into_push_pull_output_in_state(gpio::PinState::High);
    let wr = pins.gpio19.into_push_pull_output_in_state(gpio::PinState::High);
    let dc = pins.gpio20.into_push_pull_output();
    let _blk = pins.gpio28.into_push_pull_output_in_state(gpio::PinState::High);

    let lcd_d0 = pins.gpio0.into_push_pull_output();
    let lcd_d1 = pins.gpio1.into_push_pull_output();
    let lcd_d2 = pins.gpio2.into_push_pull_output();
    let lcd_d3 = pins.gpio3.into_push_pull_output();
    let lcd_d4 = pins.gpio4.into_push_pull_output();
    let lcd_d5 = pins.gpio5.into_push_pull_output();
    let lcd_d6 = pins.gpio6.into_push_pull_output();
    let lcd_d7 = pins.gpio7.into_push_pull_output();
    let lcd_d8 = pins.gpio8.into_push_pull_output();
    let lcd_d9 = pins.gpio9.into_push_pull_output();
    let lcd_d10 = pins.gpio10.into_push_pull_output();
    let lcd_d11 = pins.gpio11.into_push_pull_output();
    let lcd_d12 = pins.gpio12.into_push_pull_output();
    let lcd_d13 = pins.gpio13.into_push_pull_output();
    let lcd_d14 = pins.gpio14.into_push_pull_output();
    let lcd_d15 = pins.gpio15.into_push_pull_output();

    let bus = Generic16BitBus::new((
        lcd_d0,
        lcd_d1,
        lcd_d2,
        lcd_d3,
        lcd_d4,
        lcd_d5,
        lcd_d6,
        lcd_d7,
        lcd_d8,
        lcd_d9,
        lcd_d10,
        lcd_d11,
        lcd_d12,
        lcd_d13,
        lcd_d14,
        lcd_d15,
    ));

    let di = PGPIO16BitInterface::new(bus, dc, wr);

    let rotation = Orientation::new().rotate(Rotation::Deg270).flip_horizontal();
    let mut display = Builder::new(ILI9486Rgb565, di)
        .reset_pin(rst)
        .color_order(ColorOrder::Bgr)
        .orientation(rotation)
        .init(&mut delay)
        .unwrap();

    // Create styles used by the drawing operations.
    let light_blue = Rgb888::new(0x00, 0xd2, 0xff);
    let dark_blue = Rgb888::new(0x00, 0x0f, 0x29);
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

    display.color_converted().clear(dark_blue).unwrap();

    // Draw a 3px wide outline around the display.
    display
        .bounding_box()
        .into_styled(border_stroke)
        .draw(&mut display.color_converted()).unwrap();

    // Draw a triangle.
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(thin_stroke)
    .draw(&mut display.color_converted()).unwrap();

    // Draw a filled square
    Rectangle::new(Point::new(52, yoffset), Size::new(16, 16))
        .into_styled(fill)
        .draw(&mut display.color_converted()).unwrap();

    // Draw a circle with a 3px wide stroke.
    Circle::new(Point::new(88, yoffset), 17)
        .into_styled(thick_stroke)
        .draw(&mut display.color_converted()).unwrap();

    // Draw centered text.
    let text = "embedded-graphics";
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 15),
        character_style,
        Alignment::Center,
    )
    .draw(&mut display.color_converted()).unwrap();

    loop {}
}

/// Wrapper around `Delay` to implement the embedded-hal 1.0 delay.
///
/// This can be removed when a new version of the `cortex_m` crate is released.
struct DelayCompat(cortex_m::delay::Delay);

impl embedded_hal::delay::DelayNs for DelayCompat {
    fn delay_ns(&mut self, mut ns: u32) {
        while ns > 1000 {
            self.0.delay_us(1);
            ns = ns.saturating_sub(1000);
        }
    }

    fn delay_us(&mut self, us: u32) {
        self.0.delay_us(us);
    }

    fn delay_ms(&mut self, ms: u32) {
        self.delay_us(ms * 1000);
    }
}

// End of file
