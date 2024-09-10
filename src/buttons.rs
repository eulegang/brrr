use embassy_nrf::gpio::Input;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Sender};
use embassy_time::Instant;
use rtt_target::rprintln;

#[derive(Copy, Clone, Debug)]
pub struct ButtonEvent {
    pub button: Button,
    pub style: ButtonStyle,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Button {
    A,
    B,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ButtonStyle {
    Short,
    Long,
}

#[embassy_executor::task(pool_size = "2")]
pub async fn task(
    send: Sender<'static, ThreadModeRawMutex, ButtonEvent, 4>,
    mut input: Input<'static>,
    button: Button,
) {
    loop {
        input.wait_for_falling_edge().await;

        let pressed = Instant::now();
        input.wait_for_rising_edge().await;

        let released = Instant::now();

        let duration = released - pressed;

        let style = if duration.as_secs() >= 2 {
            ButtonStyle::Long
        } else {
            ButtonStyle::Short
        };

        let event = ButtonEvent { button, style };
        rprintln!("sending button event {:?}", event);
        let _ = send.try_send(event);
    }
}
