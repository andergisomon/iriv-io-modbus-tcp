#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::{Stack, StackResources};
use embassy_net_wiznet::chip::W5500;
use embassy_net_wiznet::*;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::SPI0;
use embassy_rp::spi::{Async, Spi};
use embassy_time::{Delay, Duration};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_io_async::Write;
use static_cell::StaticCell;
use modbus_core::*;

pub mod hal;
pub use crate::hal::*;

// NoopRawMutex, because embassy expects a rawmutex anyway. doesn't do anything, just to satisfy trait bound
#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W5500,
        SpiDevice<'static, NoopRawMutex, Spi<'static, SPI0, Async>, Output<'static>>,
        Input<'static>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let iriv_hal = hal::init(p);
    let spi = iriv_hal.w5500_spi;

    let mac_addr = [0xDE, 0xAD, 0xBE, 0xEF, 0xFE, 0xED];
    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());
    let (device, runner) = embassy_net_wiznet::new(
        mac_addr,
        state,
        iriv_hal.w5500_spi,
        iriv_hal.w5500_int,
        iriv_hal.w5500_rst,
    )
    .await
    .unwrap();

    spawner.spawn(unwrap!(ethernet_task(runner)));

    loop {
        // deal with TCP socket events here
    }
}
