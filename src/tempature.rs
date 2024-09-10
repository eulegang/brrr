use embassy_nrf::temp::Temp;
use embassy_time::Timer;
use rtt_target::rprintln;

use crate::Sender;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord)]
pub struct Temperature(pub [u8; 5]);

impl Temperature {
    pub fn iter_digits(self) -> <[u8; 5] as IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}

impl core::ops::Add for Temperature {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let mut e = self.0[4] + rhs.0[4];
        let mut d = self.0[3] + rhs.0[3];
        let mut c = self.0[2] + rhs.0[2];
        let mut b = self.0[1] + rhs.0[1];
        let mut a = self.0[0] + rhs.0[0];

        if e >= 10 {
            e %= 10;
            d += 1;
        }

        if d >= 10 {
            d %= 10;
            c += 1;
        }

        if c >= 10 {
            c %= 10;
            b += 1;
        }

        if b >= 10 {
            b %= 10;
            a += 1;
        }

        if a >= 10 {
            a = 0;
        }

        Temperature([a, b, c, d, e])
    }
}

impl core::ops::AddAssign for Temperature {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl core::ops::Sub for Temperature {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let mut acc = 0;

        let (mut e, wrap) = self.0[4].overflowing_sub(rhs.0[4] + acc);
        if wrap {
            e = (self.0[4] + 10) - (rhs.0[4] + acc);
            acc = 1;
        }

        let (mut d, wrap) = self.0[3].overflowing_sub(rhs.0[3] + acc);
        if wrap {
            d = (self.0[3] + 10) - (rhs.0[3] + acc);
            acc = 1;
        } else {
            acc = 0;
        }

        let (mut c, wrap) = self.0[2].overflowing_sub(rhs.0[2] + acc);
        if wrap {
            c = (self.0[2] + 10) - (rhs.0[2] + acc);
            acc = 1;
        } else {
            acc = 0;
        }

        let (mut b, wrap) = self.0[1].overflowing_sub(rhs.0[1] + acc);
        if wrap {
            b = (self.0[1] + 10) - (rhs.0[1] + acc);
            acc = 1;
        } else {
            acc = 0;
        }

        let (mut a, wrap) = self.0[0].overflowing_sub(rhs.0[0] + acc);
        if wrap {
            a = 0;
        }

        Temperature([a, b, c, d, e])
    }
}

impl core::ops::SubAssign for Temperature {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl core::cmp::PartialOrd for Temperature {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        for i in 0..5 {
            let x = self.0[i].partial_cmp(&other.0[i]);

            if x != Some(core::cmp::Ordering::Equal) {
                return x;
            }
        }

        return Some(core::cmp::Ordering::Equal);
    }
}

#[embassy_executor::task]
pub async fn task(send: Sender<Temperature, 1>, mut temp: Temp<'static>) {
    loop {
        let t = temp.read().await;
        let celsius = t.to_num::<f32>();
        let fahrenheit = (celsius * (9.0 / 5.0)) + 32.0;

        let calc = (fahrenheit * 10000.0) as u64;

        let e = (calc % 10) as u8;
        let d = ((calc / 10) % 10) as u8;
        let c = ((calc / 1000) % 10) as u8;
        let b = ((calc / 10000) % 10) as u8;
        let a = ((calc / 100000) % 10) as u8;

        rprintln!("sampled {} {:?}", fahrenheit, [a, b, c, d, e]);
        let _ = send.try_send(Temperature([a, b, c, d, e]));

        Timer::after_millis(5000).await;
    }
}
