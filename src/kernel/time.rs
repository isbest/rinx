use core::cmp::Ordering;
use core::fmt;
use core::fmt::Formatter;
use spin::Mutex;

use x86::io::{inb, outb};

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

/// 秒数
const CMOS_SECONDS: u8 = 0x00;
/// 分钟数
const CMOS_MINUTES: u8 = 0x02;
/// 小时数
const CMOS_HOURS: u8 = 0x04;
/// 星期数
const CMOS_CMOS_WEEKDAY: u8 = 0x06;
/// 一月的天数
const CMOS_DAY: u8 = 0x07;
/// 月数
const CMOS_MONTH: u8 = 0x08;
/// 0-99 年数(一个世纪)
const CMOS_YEAR: u8 = 0x09;
///世纪寄存器
const CMOS_CENTURY: u8 = 0x32;
/// 不可屏蔽中断
const CMOS_NMI: u8 = 0x80;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Time {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u32,
    pub century: u8,
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        self.century
            .cmp(&other.century)
            .then(self.year.cmp(&other.year))
            .then(self.month.cmp(&other.month))
            .then(self.day.cmp(&other.day))
            .then(self.hour.cmp(&other.hour))
            .then(self.minute.cmp(&other.minute))
            .then(self.second.cmp(&other.second))
    }
}

/// RTC Reader
pub struct ReadRTC {
    cmos_address: u16,
    cmos_data: u16,
    /// 当前年份,确认世纪寄存器存在可以不传
    current_year: u32,
    century_register: u8,
}

impl ReadRTC {
    #[must_use]
    pub const fn new(current_year: u32, century_register: u8) -> ReadRTC {
        ReadRTC {
            cmos_address: CMOS_ADDRESS,
            cmos_data: CMOS_DATA,
            current_year,
            century_register,
        }
    }

    // 判断是否有时间更新
    fn get_update_in_progress_flag(&mut self) -> u8 {
        unsafe {
            outb(self.cmos_address, 0x0A);
            inb(self.cmos_data) & 0x80
        }
    }

    // 获取指定的时间
    fn get_rtc_register(&mut self, reg: u8) -> u8 {
        unsafe {
            outb(self.cmos_address, reg);
            inb(self.cmos_data)
        }
    }

    /// 更新已经读取的时间
    fn update_time(&mut self) -> Time {
        // 等待时间更新结束才去读取时间
        while self.get_update_in_progress_flag() != 0 {}

        Time {
            second: self.get_rtc_register(CMOS_SECONDS),
            minute: self.get_rtc_register(CMOS_MINUTES),
            hour: self.get_rtc_register(CMOS_HOURS),
            day: self.get_rtc_register(CMOS_DAY),
            month: self.get_rtc_register(CMOS_MONTH),
            year: self.get_rtc_register(CMOS_YEAR) as u32,
            century: self.get_rtc_register(self.century_register),
        }
    }

    /// bcd转换成二进制
    #[inline(always)]
    fn bcd_to_bin(bcd: u8) -> u8 {
        (bcd & 0xf) + (bcd >> 4) * 10
    }

    pub fn read(&mut self) -> Time {
        let mut last_time: Time;
        let mut time: Time = self.update_time();

        // 循环读取,直到误差为1s
        loop {
            last_time = time;
            time = self.update_time();

            if (last_time.second == time.second)
                && (last_time.minute == time.minute)
                && (last_time.hour == time.hour)
                && (last_time.day == time.day)
                && (last_time.month == time.month)
                && (last_time.year == time.year)
                && (last_time.century == time.century)
            {
                break;
            }
        }

        let register_b = self.get_rtc_register(0x0B);

        // 编码则将bcd转换成二进制
        if register_b & 0x04 == 0 {
            time.second = ReadRTC::bcd_to_bin(time.second);
            time.minute = ReadRTC::bcd_to_bin(time.minute);
            time.hour =
                ((time.hour & 0x0F) + (((time.hour & 0x70) / 16) * 10)) | (time.hour & 0x80);
            time.day = ReadRTC::bcd_to_bin(time.day);
            time.month = ReadRTC::bcd_to_bin(time.month);
            time.year = ReadRTC::bcd_to_bin(time.year as u8) as u32;

            if self.century_register != 0 {
                time.century = ReadRTC::bcd_to_bin(time.century);
            }
        }

        // 12小时转换成24小时
        if register_b & 0x02 == 0 && (time.hour & 0x80 != 0) {
            time.hour = ((time.hour & 0x7F) + 12) % 24;
        }

        // 世纪寄存器
        if self.century_register == 0 {
            // 世纪寄存器不存在,就用传入的年份,计算世纪
            time.year += (self.current_year / 100) * 100;

            if time.year < self.current_year {
                time.year += 100;
            };
        } else {
            // 世纪乘上100,再加上年份
            time.year += (time.century as u32) * 100;
        }

        time
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[TIME] STARTUP TIME: {:02}-{:02}-{:02} {:02}:{:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute, self.second
        )
    }
}

static RTC: Mutex<ReadRTC> = Mutex::new(ReadRTC::new(2023, CMOS_CENTURY));

pub fn now_time() -> Time {
    RTC.lock().read()
}
