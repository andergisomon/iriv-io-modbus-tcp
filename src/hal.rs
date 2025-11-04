/// IRIV IOC HAL
/// Ported from the MicroPython impl: https://github.com/CytronTechnologies/Cytron-IRIV-IO-Controller/blob/main/examples/circuitpython/modbus_io_expander/source/lib/iriv_ioc_hal.py
/// Keeping things simple for now: Pins with counter function not implemented

use core::sync::atomic::{AtomicU32, Ordering};
use embassy_rp::adc::{Adc, Channel};
use embassy_rp::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_rp::peripherals::*;
use embassy_rp::spi::{self, Spi, Config as SpiConfig};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embassy_executor::Spawner;

pub const SUPPLY_VOLTAGE_MV: u32 = 3320;

pub struct Hal {
    pub led: Output<'static, PIN_25>,
    pub dout: [Output<'static, AnyPin>; 4],
    pub din: [Input<'static, AnyPin>; 11],
    pub adc: Adc,
    pub an0: PIN_26,
    pub an1: PIN_27,
    pub w5500_spi: Spi<'static, SPI0, spi::Blocking>,
    pub w5500_cs: Output<'static, PIN_5>,
    pub w5500_rst: Output<'static, PIN_6>,
}

bind_interrupts!(
    struct Irqs {
        ADC_IRQ_FIFO => InterruptHandler;
    }
);

pub fn init(p: embassy_rp::Peripherals) -> Hal {
    // MicroPython should map this to GPIO25, but the IRIV IOC datasheet reserves that to the RS-485 LED
    // Cytron leaves GPIO29 free for the user-defined USR LED
    let led = Output::new(p.PIN_25, Level::Low);

    // DO0–DO3
    let dout = [
        Output::new(p.PIN_0.degrade(), Level::Low),
        Output::new(p.PIN_1.degrade(), Level::Low),
        Output::new(p.PIN_2.degrade(), Level::Low),
        Output::new(p.PIN_3.degrade(), Level::Low),
    ];

    // DI0–DI10
    let din = [
        Input::new(p.PIN_4.degrade(), Pull::None),
        Input::new(p.PIN_7.degrade(), Pull::None),
        Input::new(p.PIN_8.degrade(), Pull::None),
        Input::new(p.PIN_9.degrade(), Pull::None),
        Input::new(p.PIN_10.degrade(), Pull::None),
        Input::new(p.PIN_11.degrade(), Pull::None),
        Input::new(p.PIN_12.degrade(), Pull::None),
        Input::new(p.PIN_13.degrade(), Pull::None),
        Input::new(p.PIN_14.degrade(), Pull::None),
        Input::new(p.PIN_15.degrade(), Pull::None),
        Input::new(p.PIN_16.degrade(), Pull::None),
    ];

    let cfg = SpiConfig::default();
    cfg.frequency = 40_000_000; // w5500 supports up to 50MHz, and the RP2350 can technically do more, but let's just play it safe
    let spi = Spi::new(
        p.SPI0,
        p.PIN_22,
        p.PIN_19,
        p.PIN_20,
        p.DMA_CH0,
        p.DMA_CH1,
        cfg
    );
    let w5500_cs = Output::new(p.PIN_5, Level::High);
    let w5500_rst = Output::new(p.PIN_6, Level::High);

    let adc = Adc::new(p.ADC, Irqs, Config::default());
    let an0 = p.PIN_26;
    let an1 = p.PIN_27;

    Hal {
        led,
        dout,
        din,
        adc,
        an0,
        an1,
        w5500_spi: spi,
        w5500_cs,
        w5500_rst,
    }
}

// TODO: embed units into the type system
pub fn an_read_voltage_mv(hal: &mut Hal, channel: usize) -> u32 {
    let val = match channel {
        0 => hal.adc.read(&mut hal.an0),
        1 => hal.adc.read(&mut hal.an1),
        _ => 0,
    } as f32;
    (val * (SUPPLY_VOLTAGE_MV as f32 / 4095.0 * 16.0 / 5.0)) as u32
}

// TODO: embed units into the type system
pub fn an_read_current_ua(hal: &mut Hal, channel: usize) -> u32 {
    let val = match channel {
        0 => hal.adc.read(&mut hal.an0),
        1 => hal.adc.read(&mut hal.an1),
        _ => 0,
    } as f32;
    (val * (SUPPLY_VOLTAGE_MV as f32 / 4095.0 * 16.0 / 5.0 / 248.0 * 1000.0)) as u32
}
