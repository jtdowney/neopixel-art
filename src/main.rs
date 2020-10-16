#![no_std]
#![no_main]

use cortex_m_rt::entry;
use hal::delay::Delay;
use hal::prelude::*;
use hal::stm32;
use hal::time::MegaHertz;
use hal::timer::Timer;
use panic_reset as _;
use smart_leds::{self, SmartLedsWrite, RGB8};
use stm32f1xx_hal as hal;
use ws2812_timer_delay::Ws2812;

const MAX_BRIGHTNESS: u8 = 20;
const WIDTH: usize = 16;
const PIXELS: usize = WIDTH * WIDTH;
const IMAGE: &[u8] = include_bytes!("../pixel-art.rgb");

fn create_screen() -> [RGB8; PIXELS] {
    let mut screen: [RGB8; PIXELS] = [RGB8::default(); PIXELS];
    let mut index = 0;
    for j in (0..WIDTH).rev() {
        let mut step: isize = 1;
        let mut i = 0;

        // the panel I have snakes around at the edge
        if j % 2 == 0 {
            step = -1;
            i = WIDTH - 1;
        }

        while i < WIDTH {
            let offset = (j * WIDTH * 3) + (i * 3);
            let r = IMAGE[offset];
            let g = IMAGE[offset + 1];
            let b = IMAGE[offset + 2];

            screen[index] = RGB8::new(r, g, b);

            index += 1;
            i = (i as isize + step) as usize;
        }
    }

    screen
}

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let timer = Timer::tim1(dp.TIM1, &clocks, &mut rcc.apb2).start_count_down(MegaHertz(3));
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    let data_pin = gpioa.pa7.into_push_pull_output(&mut gpioa.crl);
    let mut ws = Ws2812::new(timer, data_pin);

    let screen = create_screen();

    let brightness = (0..MAX_BRIGHTNESS).chain((0..MAX_BRIGHTNESS).rev()).cycle();
    for b in brightness {
        let data = smart_leds::brightness(screen.iter().cloned(), b);
        ws.write(data).unwrap();
        delay.delay_ms(50 as u16);
    }

    loop {}
}
