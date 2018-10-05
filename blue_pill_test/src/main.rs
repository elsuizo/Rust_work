//! Open loop motor control

#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

#[macro_use]
extern crate cortex_m_rt as rt;
extern crate cortex_m_semihosting as sh;
extern crate motor_driver;
extern crate panic_semihosting;
#[macro_use(block)]
extern crate nb;
extern crate stm32f103xx_hal as hal;

use core::fmt::Write;
use hal::delay::Delay;
use hal::prelude::*;
use hal::serial::Serial;
use motor_driver::Motor;
use sh::hio;


use hal::prelude::*;
use hal::stm32f103xx;
use hal::timer::Timer;
use rt::ExceptionFrame;

entry!(main);

fn main() -> ! {
    let p = stm32f103xx::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut afio = p.AFIO.constrain(&mut rcc.apb2);

    let mut delay = Delay::new(cp.SYST, clocks);
    let mut gpioa = p.GPIOA.split(&mut rcc.apb2);

    // para el stby
    let mut stby = gpioa.pa3.into_push_pull_output(&mut gpioa.crl);
    stby.set_high();


    let pwm = p.TIM2.pwm(
        gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl),
        &mut afio.mapr,
        1.khz(),
        clocks,
        &mut rcc.apb1,
    );

    let max_duty = pwm.get_max_duty() as i16;
    let mut motor = Motor::tb6612fng(
        gpioa.pa1.into_push_pull_output(&mut gpioa.crl),
        gpioa.pa2.into_push_pull_output(&mut gpioa.crl),
        pwm,
    );

    let mut duty = max_duty / 5;
    let mut brake = true;
    motor.duty(duty as u16);

    loop {
        for index in (0..max_duty).step_by(50) {
            motor.duty(index as u16);
            delay.delay_ms(70_u16);
            motor.cw();
        }
        for index in (0..max_duty).step_by(50) {
            motor.duty(index as u16);
            delay.delay_ms(30_u16);
            motor.ccw();
        }
    }

}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
