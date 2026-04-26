#![no_main]
#![no_std]

use common as _;

use stm32f3xx_hal as hal;

use cortex_m_rt::entry;
//use embedded_hal::digital::OutputPin;
use hal::{
    gpio::{Gpioe, Output, Pin, PushPull, Ux},
    pac,
    prelude::*,
};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    let led0 = gpioe
        .pe8
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led1 = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led2 = gpioe
        .pe10
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led3 = gpioe
        .pe11
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led4 = gpioe
        .pe12
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led5 = gpioe
        .pe13
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led6 = gpioe
        .pe14
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);
    let led7 = gpioe
        .pe15
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    let mut leds: [Pin<Gpioe, Ux, Output<PushPull>>; 8] = [
        led0.downgrade(),
        led1.downgrade(),
        led2.downgrade(),
        led3.downgrade(),
        led4.downgrade(),
        led5.downgrade(),
        led6.downgrade(),
        led7.downgrade(),
    ];

    for led in &mut leds {
        led.set_low().unwrap();
    }

    loop {
        for i in 0..8 {
            if leds[i].is_set_low().unwrap() {
                leds[i].set_high().unwrap();
            } else {
                leds[i].set_low().unwrap();
            }
            cortex_m::asm::delay(1_000_000);
        }
    }
}
