/// IRIV IOC HAL
/// Ported from the MicroPython impl: https://github.com/CytronTechnologies/Cytron-IRIV-IO-Controller/blob/main/examples/circuitpython/modbus_io_expander/source/lib/iriv_ioc_hal.py
/// Keeping things simple for now: Pins with counter function not implemented

use embassy_rp::adc::{Adc, Async, Channel, Config, InterruptHandler};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::{Peri, Peripherals, peripherals::*};
use embassy_rp::spi::{self, Spi, Config as SpiConfig};
use embassy_rp::bind_interrupts;
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, RawMutex};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

pub const SUPPLY_VOLTAGE_MV: u32 = 3320;

pub struct Io {
    pub dout: [Output<'static>; 4],
    pub din: [Input<'static>; 11],
    pub an0: Channel<'static>,
    pub an1: Channel<'static>,
    pub adc_dma_channel: Peri<'static, DMA_CH2>,
    pub adc: Adc<'static, Async>,
}

// TODO: Just create channels and put them in the Hal struct for the I/O pins
pub struct Hal {
    pub led: Output<'static>,
    pub io: Io,
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

    // AN0-AN1
    let an0 = Channel::new_pin(p.PIN_26, Pull::None);
    let an1 = Channel::new_pin(p.PIN_27, Pull::None);

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
    // let adc_dma_channel = Mutex::new( p.DMA_CH2)
    let adc_dma_channel: Peri<'static, DMA_CH2> = p.DMA_CH2;

    let io = Io {
        dout,
        din,
        an0,
        an1,
        adc_dma_channel,
        adc
    };

    Hal {
        led,
        io,
        w5500_spi: spi,
        w5500_int,
        // w5500_cs,
        w5500_rst,
    }
}

// TODO: embed units into the type system
pub async fn an_read_voltage_mv(hal: &mut Io, channel: usize) -> u32 {
    let pin_0 = &mut hal.an0;
    let pin_1 = &mut hal.an1;
    let mut buf = [0_u16; 64];
    let adc_dma = hal.adc_dma_channel.reborrow();

    let val = match channel {
        0 => {
            // 100kS/s sample rate
            hal.adc.read_many(pin_0, &mut buf, 479, adc_dma).await.unwrap();
            buf.iter().map(|&x| x as f32).sum::<f32>() / buf.len() as f32
        },
        1 => {
            // 100kS/s sample rate
            hal.adc.read_many(pin_1, &mut buf, 479, adc_dma).await.unwrap();
            buf.iter().map(|&x| x as f32).sum::<f32>() / buf.len() as f32
        },
        _ => 0.0,
    } as f32;
    (val * (SUPPLY_VOLTAGE_MV as f32 / 65535.0 * 16.0 / 5.0)) as u32
}

// TODO: embed units into the type system
pub async fn an_read_current_ua(hal: &mut Io, channel: usize) -> u32 {
    let pin_0 = &mut hal.an0;
    let pin_1 = &mut hal.an1;
    let mut buf = [0_u16; 64];
    let adc_dma = hal.adc_dma_channel.reborrow();

    let val = match channel {
        0 => {
            // 100kS/s sample rate
            hal.adc.read_many(pin_0, &mut buf, 479, adc_dma).await.unwrap();
            buf.iter().map(|&x| x as f32).sum::<f32>() / buf.len() as f32
        },
        1 => {
            // 100kS/s sample rate
            hal.adc.read_many(pin_1, &mut buf, 479, adc_dma).await.unwrap();
            buf.iter().map(|&x| x as f32).sum::<f32>() / buf.len() as f32
        },
        _ => 0.0,
    } as f32;
    (val * (SUPPLY_VOLTAGE_MV as f32 / 65535.0 * 16.0 / 5.0 / 248.0 * 1000.0)) as u32
}
