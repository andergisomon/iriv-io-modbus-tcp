/// IRIV IOC HAL
/// Ported from the MicroPython impl: https://github.com/CytronTechnologies/Cytron-IRIV-IO-Controller/blob/main/examples/circuitpython/modbus_io_expander/source/lib/iriv_ioc_hal.py
/// Keeping things simple for now: Pins with counter function not implemented

use embassy_rp::adc::{Adc, Async, Channel, Config, InterruptHandler};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::{Peri, Peripherals, peripherals::*};
use embassy_rp::spi::{self, Spi, Config as SpiConfig};
use embassy_rp::bind_interrupts;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

pub const SUPPLY_VOLTAGE_MV: u32 = 3320;

// TODO: Just create channels and put them in the Hal struct for the I/O pins
pub struct Hal {
    pub led: Output<'static>,
    pub dout: [Output<'static>; 4],
    pub din: [Input<'static>; 11],
    pub adc: Adc<'static, Async>,
    pub an0: Peri<'static, PIN_26>,
    pub an1: Peri<'static, PIN_27>,
    pub w5500_int: Input<'static>,
    pub w5500_spi: SpiDevice<'static, NoopRawMutex, Spi<'static, SPI0, spi::Async>, Output<'static>>,
    // pub w5500_cs: Output<'static>,
    pub w5500_rst: Output<'static>,
}

bind_interrupts!(
    struct Irqs {
        ADC_IRQ_FIFO => InterruptHandler;
    }
);

pub fn init(p: embassy_rp::Peripherals) -> Hal {
    // MicroPython should map this to GPIO25, but the IRIV IOC datasheet reserves that to the RS-485 LED
    // Cytron leaves GPIO29 free for the user-defined USR LED
    let led = Output::new(p.PIN_29, Level::Low);

    // DO0–DO3
    let dout = [
        Output::new(p.PIN_0, Level::Low),
        Output::new(p.PIN_1, Level::Low),
        Output::new(p.PIN_2, Level::Low),
        Output::new(p.PIN_3, Level::Low),
    ];

    // DI0–DI10)
    let din = [
        Input::new(p.PIN_4, Pull::None),
        Input::new(p.PIN_7, Pull::None),
        Input::new(p.PIN_8, Pull::None),
        Input::new(p.PIN_9, Pull::None),
        Input::new(p.PIN_10, Pull::None),
        Input::new(p.PIN_11, Pull::None),
        Input::new(p.PIN_12, Pull::None),
        Input::new(p.PIN_13, Pull::None),
        Input::new(p.PIN_14, Pull::None),
        Input::new(p.PIN_15, Pull::None),
        Input::new(p.PIN_16, Pull::None),
    ];

    let mut cfg = SpiConfig::default();
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

    let w5500_int = Input::new(p.PIN_18, Pull::Up);
    let w5500_cs = Output::new(p.PIN_5, Level::High);
    let w5500_rst = Output::new(p.PIN_6, Level::High);

    static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<'static, SPI0, spi::Async>>> = StaticCell::new();
    let spi_bus = SPI_BUS.init(Mutex::new(spi));

    let spi = SpiDevice::new(spi_bus, w5500_cs);

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
        w5500_int,
        // w5500_cs,
        w5500_rst,
    }
}

// WIP
// TODO: Just create channels and put them in the Hal struct for the I/O pins
// TODO: embed units into the type system
pub async fn an_read_voltage_mv(p: Peripherals, hal: &mut Hal, channel: usize) -> u32 {
    let mut pin_0 = Channel::new_pin(hal.an0, Pull::None);
    let mut pin_1 = Channel::new_pin(hal.an1, Pull::None);
    let mut buf = [0_u16; 64];

    let val = match channel {
        0 => {
            hal.adc.read_many(&mut pin_0, &mut buf, 479, p.DMA_CH2).await.unwrap();
            buf.iter().map(|&x| x as f32).sum::<f32>() / buf.len() as f32
        },
        1 => {
            hal.adc.read_many(&mut pin_1, &mut buf, 479, p.DMA_CH2).await.unwrap();
            buf.iter().map(|&x| x as f32).sum::<f32>() / buf.len() as f32
        },
        _ => 0.0,
    } as f32;
    (val * (SUPPLY_VOLTAGE_MV as f32 / 4095.0 * 16.0 / 5.0)) as u32
}

// TODO: embed units into the type system
// pub fn an_read_current_ua(hal: &mut Hal, channel: usize) -> u32 {
//     let val = match channel {
//         0 => hal.adc.read_many(&mut hal.an0),
//         1 => hal.adc.read_many(&mut hal.an1),
//         _ => 0,
//     } as f32;
//     (val * (SUPPLY_VOLTAGE_MV as f32 / 4095.0 * 16.0 / 5.0 / 248.0 * 1000.0)) as u32
// }
