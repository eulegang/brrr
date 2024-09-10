use embassy_nrf::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Receiver};
use embassy_time::Timer;
use futures::{select_biased, FutureExt};
use rtt_target::rprintln;

use crate::tempature::Temperature;

pub struct Matrix {
    cols: [Output<'static>; 5],
    rows: [Output<'static>; 5],
}

pub struct Frame {
    pixel: [[bool; 5]; 5],
}

fn sel_out<const N: usize>(index: usize, pins: &mut [Output<'static>; N]) {
    for (i, pin) in pins.iter_mut().enumerate() {
        let level = if index == i { Level::High } else { Level::Low };
        pin.set_level(level);
    }
}

fn map_out<const N: usize>(mask: &[bool], pins: &mut [Output<'static>; N]) {
    for (mask, pin) in mask.iter().zip(pins) {
        let level = if *mask { Level::Low } else { Level::High };
        pin.set_level(level);
    }
}

impl Matrix {
    pub fn new(cols: [Output<'static>; 5], rows: [Output<'static>; 5]) -> Matrix {
        Matrix { cols, rows }
    }

    pub async fn print(&mut self, frame: &Frame) {
        for (y, row) in frame.pixel.iter().enumerate() {
            sel_out(y, &mut self.rows);
            map_out(row, &mut self.cols);
            Timer::after_micros(2000).await;
        }
    }
}

pub enum UI {
    Temp(Temperature, bool),
    SetAlarm(Temperature),
    Alarm,
}

impl UI {
    fn print(&self, frame: &mut Frame) {
        match self {
            UI::Temp(temp, muted) => {
                frame.pixel[0] = [true, false, false, false, *muted];

                for (row, bit) in [0x8, 0x4, 0x2, 0x1].iter().enumerate() {
                    for (i, val) in temp.iter_digits().enumerate() {
                        let val = val & bit != 0;

                        frame.pixel[row + 1][i] = val;
                    }
                }
            }

            UI::SetAlarm(alarm) => {
                rprintln!("alarm: {:?}", alarm);
                frame.pixel[0] = [false, true, false, false, false];

                for (row, bit) in [0x8, 0x4, 0x2, 0x1].iter().enumerate() {
                    for (i, val) in alarm.iter_digits().enumerate() {
                        let val = val & bit != 0;

                        frame.pixel[row + 1][i] = val;
                    }
                }
            }

            UI::Alarm => {
                frame.pixel[0] = [false, true, false, false, false];

                for i in 1..5 {
                    frame.pixel[i] = [true, true, true, true, true];
                }
            }
        }
    }
}

#[embassy_executor::task]
pub async fn task(mut matrix: Matrix, recv: Receiver<'static, ThreadModeRawMutex, UI, 5>) {
    let mut frame = Frame {
        pixel: [
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
            [false, false, false, false, false],
        ],
    };

    matrix.print(&frame).await;

    loop {
        select_biased! {
            ui = recv.receive().fuse() => {
                ui.print(&mut frame);
            }

            _ = Timer::after_millis(1).fuse() => {
                matrix.print(&frame).await;
            }
        }
    }
}
