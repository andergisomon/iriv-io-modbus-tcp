#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_net::{Ipv4Cidr, Stack, StackResources};
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
use embassy_rp::clocks::RoscRng;

pub mod hal;
pub use crate::hal::*;
pub mod modbus;
pub use crate::modbus::*;

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
async fn ip_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut iriv_hal = hal::init(p);

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
    spawner.spawn(ethernet_task(runner)).unwrap();

    let net_config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(embassy_net::Ipv4Address::new(192, 168, 1, 123), 24),
        gateway: None,
        dns_servers: Default::default()
    });

    let mut rng = RoscRng;
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

    let (stack, runner) = embassy_net::new(
        device,
        net_config,
        RESOURCES.init(StackResources::new()),
        rng.next_u64(),
    );
    spawner.spawn(ip_task(runner)).unwrap();

    let mut rx_buffer = [0; 32];
    let mut tx_buffer = [0; 32];
    let mut buf = [0; 1024];

    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(12)));

        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }
        transact_client(&mut buf, &mut iriv_hal.io, socket).await;
    }
}
