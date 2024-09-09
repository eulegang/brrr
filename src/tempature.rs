use embassy_nrf::temp::Temp;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Sender};
use embassy_time::Timer;
use rtt_target::rprintln;

#[embassy_executor::task]
pub async fn task(send: Sender<'static, ThreadModeRawMutex, f32, 1>, mut temp: Temp<'static>) {
    loop {
        let t = temp.read().await;
        let celsius = t.to_num::<f32>();
        let fahrenheit = (celsius * (9.0 / 5.0)) + 32.0;

        rprintln!("sampled {}", fahrenheit);
        //let _ = send.try_send(fahrenheit);

        Timer::after_millis(5000).await;
    }
}
