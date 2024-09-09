use embassy_nrf::gpio::Output;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Receiver};

const DURATION_MS: i32 = 16;

pub struct Matrix {
    cols: [Output<'static>; 5],
    rows: [Output<'static>; 5],
}

pub struct Frame {
    pixel: [[bool; 5]; 5],
}

impl Matrix {
    pub fn new(cols: [Output<'static>; 5], rows: [Output<'static>; 5]) -> Matrix {
        Matrix { cols, rows }
    }

    pub async fn print(&self, frame: &Frame) {
        // TODO: something more intelligent with timers

        /*
                let delay = 2;
                let loops = DURATION_MS / (self.rows.len() as u32 * delay);
                for _ in 0..loops {
                    for (row_line, led_matrix_row) in self.rows.iter_mut().zip(led_matrix.iter()) {
                        row_line.set_high().ok();
                        for (col_line, led_matrix_val) in self.cols.iter_mut().zip(led_matrix_row.iter()) {
                            // TODO : use value to set brightness
                            if *led_matrix_val > 0 {
                                col_line.set_low().ok();
                            }
                        }
                        delay.delay_us(self.delay_ms * 1000);
                        for col_line in &mut self.cols {
                            col_line.set_high().ok();
                        }
                        row_line.set_low().ok();
                    }
                }
        */
    }
}

pub enum UI {}

#[embassy_executor::task]
pub async fn task(matrix: Matrix, recv: Receiver<'static, ThreadModeRawMutex, UI, 5>) {
    loop {
        let r = recv.receive().await;
    }
}
