use futures::{select_biased, FutureExt as _};
use rtt_target::rprintln;

use crate::{
    buttons::{Button, ButtonEvent, ButtonStyle},
    display::UI,
    tempature::Temperature,
    Recver, Sender,
};

#[derive(Debug)]
struct State {
    mode: Mode,
    temp: Temperature,
    alarm: Temperature,
    muted: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    Base,
    SetAlarm,
}

impl Mode {
    fn inc(self) -> Self {
        match self {
            Mode::Base => Mode::SetAlarm,
            Mode::SetAlarm => Mode::Base,
        }
    }

    fn dec(self) -> Self {
        match self {
            Mode::Base => Mode::SetAlarm,
            Mode::SetAlarm => Mode::Base,
        }
    }
}

impl State {
    fn handle_temp(&mut self, temp: Temperature, ui: &Sender<UI, 5>) {
        rprintln!("sending temp");

        if self.mode == Mode::Base {
            let _ = ui.try_send(UI::Temp(temp, self.muted));
        }

        self.temp = temp;
    }

    fn handle_button(&mut self, button: ButtonEvent, ui: &Sender<UI, 5>) {
        match button.style {
            ButtonStyle::Long => {
                match button.button {
                    Button::A => self.mode = self.mode.dec(),
                    Button::B => self.mode = self.mode.inc(),
                }

                let _ = match self.mode {
                    Mode::Base => {
                        if self.temp < self.alarm && !self.muted {
                            ui.try_send(UI::Alarm)
                        } else {
                            ui.try_send(UI::Temp(self.temp, self.muted))
                        }
                    }
                    Mode::SetAlarm => ui.try_send(UI::SetAlarm(self.alarm)),
                };
            }

            ButtonStyle::Short => match self.mode {
                Mode::Base => {
                    if button.button == Button::B {
                        self.muted = !self.muted;
                    }

                    if self.temp < self.alarm && !self.muted {
                        let _ = ui.try_send(UI::Alarm);
                    } else {
                        let _ = ui.try_send(UI::Temp(self.temp, self.muted));
                    }
                }

                Mode::SetAlarm => {
                    match button.button {
                        Button::A => {
                            self.alarm -= Temperature([0, 0, 1, 0, 0]);
                            self.alarm = self.alarm.max(Temperature([5, 5, 0, 0, 0]));
                        }

                        Button::B => {
                            self.alarm += Temperature([0, 0, 1, 0, 0]);
                            self.alarm = self.alarm.min(Temperature([9, 9, 9, 0, 0]));
                        }
                    };

                    let _ = ui.try_send(UI::SetAlarm(self.alarm));
                }
            },
        }

        rprintln!("button pressed");
    }
}

#[embassy_executor::task]
pub async fn task(
    temp: Recver<Temperature, 1>,
    buttons: Recver<ButtonEvent, 4>,
    ui: Sender<UI, 5>,
) {
    let mut state = State {
        mode: Mode::Base,
        temp: Temperature([0u8; 5]),
        alarm: Temperature([7u8, 0u8, 0u8, 0u8, 0u8]),
        muted: false,
    };

    loop {
        select_biased! {
            temp = temp.receive().fuse() => state.handle_temp(temp, &ui),
            button = buttons.receive().fuse() => state.handle_button(button, &ui),
        }
    }
}
