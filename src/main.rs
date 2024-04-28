#![no_std]
#![no_main]

mod task;

use bsp::{
    entry,
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        Sio, Watchdog,
    },
    pac,
};
use defmt::*;
use embedded_hal::digital::OutputPin;
use rp_pico as bsp;
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    info!("init board");

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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    stack_start();

    let mut led_pin = pins.led.into_push_pull_output();

    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(500);

        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}

extern "C" {
    static _stack_start: u8;
}
fn stack_start() {
    let stack_start = unsafe { &_stack_start as *const _ as usize };
    info!("address of _stack_start: {:?}", stack_start);
}
