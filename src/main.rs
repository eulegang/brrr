#![no_std]
#![no_main]

use buttons::Button;
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive},
    temp::{self, Temp},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

bind_interrupts!(struct Irqs {
    TEMP => temp::InterruptHandler;
});

mod app;
mod buttons;
mod display;
mod tempature;

pub type Chan<T, const N: usize> = embassy_sync::channel::Channel<ThreadModeRawMutex, T, N>;
pub type Recver<T, const N: usize> =
    embassy_sync::channel::Receiver<'static, ThreadModeRawMutex, T, N>;
pub type Sender<T, const N: usize> =
    embassy_sync::channel::Sender<'static, ThreadModeRawMutex, T, N>;

static TEMP_CHAN: Chan<tempature::Temperature, 1> = Channel::new();
static UI_CHAN: Chan<display::UI, 5> = Channel::new();
static BUTTON_CHAN: Chan<buttons::ButtonEvent, 4> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    rtt_init_print!();

    let p = embassy_nrf::init(Default::default());

    let temp = Temp::new(p.TEMP, Irqs);
    let matrix = display::Matrix::new(
        [
            Output::new(p.P0_28, Level::High, OutputDrive::Standard),
            Output::new(p.P0_11, Level::High, OutputDrive::Standard),
            Output::new(p.P0_31, Level::High, OutputDrive::Standard),
            Output::new(p.P1_05, Level::High, OutputDrive::Standard),
            Output::new(p.P0_30, Level::High, OutputDrive::Standard),
        ],
        [
            Output::new(p.P0_21, Level::High, OutputDrive::Standard),
            Output::new(p.P0_22, Level::High, OutputDrive::Standard),
            Output::new(p.P0_15, Level::High, OutputDrive::Standard),
            Output::new(p.P0_24, Level::High, OutputDrive::Standard),
            Output::new(p.P0_19, Level::High, OutputDrive::Standard),
        ],
    );

    spawner
        .spawn(tempature::task(TEMP_CHAN.sender(), temp))
        .unwrap();

    spawner
        .spawn(buttons::task(
            BUTTON_CHAN.sender(),
            Input::new(p.P0_14, embassy_nrf::gpio::Pull::None),
            Button::A,
        ))
        .unwrap();

    spawner
        .spawn(buttons::task(
            BUTTON_CHAN.sender(),
            Input::new(p.P0_23, embassy_nrf::gpio::Pull::None),
            Button::B,
        ))
        .unwrap();

    spawner
        .spawn(display::task(matrix, UI_CHAN.receiver()))
        .unwrap();

    spawner
        .spawn(app::task(
            TEMP_CHAN.receiver(),
            BUTTON_CHAN.receiver(),
            UI_CHAN.sender(),
        ))
        .unwrap();
}
