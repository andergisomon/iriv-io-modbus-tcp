use defmt::*;
use embassy_net::tcp::TcpSocket;
use embassy_rp::gpio::{Level};
use modbus_core::tcp::{RequestAdu, ResponseAdu};
use modbus_core::tcp::server::{decode_request, encode_response};
use modbus_core::{Coils, Data, Error, Request, RequestPdu, Response, ResponsePdu};
use crate::Io;

#[derive(Debug)]
pub enum CallbackError {
    NoAddressMatch
}

/// Register addresses
/// Coils (0x0) - Read/Write
const DO0_ADDR: u16 = 0x0100; /// Digital Output 0
const DO1_ADDR: u16 = 0x0101; /// Digital Output 1
const DO2_ADDR: u16 = 0x0102; /// Digital Output 2
const DO3_ADDR: u16 = 0x0103; /// Digital Output 3
const MAX_NUMBER_OF_COILS: usize = 4;

/// Contacts (0x1) - Read-only
const DI0_ADDR:  u16 = 0x0000; /// Digital Input 0
const DI1_ADDR:  u16 = 0x0001; /// Digital Input 1
const DI2_ADDR:  u16 = 0x0002; /// Digital Input 2
const DI3_ADDR:  u16 = 0x0003; /// Digital Input 3
const DI4_ADDR:  u16 = 0x0004; /// Digital Input 4
const DI5_ADDR:  u16 = 0x0005; /// Digital Input 5
const DI6_ADDR:  u16 = 0x0006; /// Digital Input 6
const DI7_ADDR:  u16 = 0x0007; /// Digital Input 7
const DI8_ADDR:  u16 = 0x0008; /// Digital Input 8
const DI9_ADDR:  u16 = 0x0009; /// Digital Input 9
const DI10_ADDR: u16 = 0x000a; /// Digital Input 10

/// Input Registers (3x) - Read-only
const ANV0_ADDR: u16 = 0x0200; /// Analog Input 0 (V)
const ANV1_ADDR: u16 = 0x0201; /// Analog Input 1 (V)
const ANA0_ADDR: u16 = 0x0210; /// Analog Input 0 (mA)
const ANA1_ADDR: u16 = 0x0211; /// Analog Input 1 (mA)

const MODEL1_ADDR: u16 = 0x0f00; /// Model Name 1 (Read-only)
const MODEL2_ADDR: u16 = 0x0f01; /// Model Name 2 (Read-only)

const VERSION_MAJOR_ADDR: u16 = 0x0f10; /// Major Version (Read-only)
const VERSION_MINOR_ADDR: u16 = 0x0f11; /// Minor Version (Read-only)
const VERSION_PATCH_ADDR: u16 = 0x0f12; /// Patch Version (Read-only)

/// Model name
const MODEL1_VAL: u16 = 0x494f;
const MODEL2_VAL: u16 = 0x4300;

/// Firmware Version, should be synced with Cargo semver (major.minor.patch)
const VERSION_MAJOR_VAL: u16 = 0;
const VERSION_MINOR_VAL: u16 = 1;
const VERSION_PATCH_VAL: u16 = 0;

/// Services client reads and writes
/// TCP socket timeout is set in main()
/// modbus-core has yet to implement modbus exception responses
pub async fn transact_client(buf: &mut [u8], hal: &mut Io, socket: TcpSocket<'_>) -> Result<(), Error> {

    loop {
        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                warn!("read EOF");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                warn!("{:?}", e);
                break;
            }
        };
        info!("received {:?}", &buf[..n]);
    }

    let req = decode_request(buf)?.unwrap();

    // handle what kind of modbus request it is
    // just use match statements here to handle client read/writes
    let RequestAdu {hdr: header, pdu: req_data} = req;

    let mut resp_data;
    let mut resp;
    let target_buf = &mut [0u8; MAX_NUMBER_OF_COILS];

    match req_data {
        RequestPdu(Request::ReadCoils(addr, quantity)) => {
            let bools = &mut [false; MAX_NUMBER_OF_COILS];
            for i in 0..quantity as usize {
                bools[i] = dout_get_cb(i, &hal);
            }

            let start_addr: Result<usize, CallbackError> = match addr {
                DO0_ADDR => Ok(0),
                DO1_ADDR => Ok(1),
                DO2_ADDR => Ok(2),
                DO3_ADDR => Ok(3),
                _ => {
                    error!("Modbus address given does not match any register definition");
                    Err(CallbackError::NoAddressMatch)
                }
            };

            let truncated_bools = &mut bools[start_addr.unwrap_or_else(|_| -> usize {0})..];

            let coils = Coils::from_bools(truncated_bools, target_buf).unwrap();
            resp_data = ResponsePdu(Ok(Response::ReadCoils(coils)));
            resp = ResponseAdu {hdr: header, pdu: resp_data}; // copy MBAP header, put data to service request
        },
        RequestPdu(Request::ReadInputRegisters(addr, quantity)) => {

            // handle requests here

            resp_data = ResponsePdu(Ok(Response::ReadInputRegisters(Data::from_words(&[0], target_buf).unwrap())));
            resp = ResponseAdu {hdr: header, pdu: resp_data}; // copy MBAP header, put data to service request
        },
        RequestPdu(Request::WriteSingleCoil(addr, val)) => {

            // handle requests here

            resp_data = ResponsePdu(Ok(Response::WriteSingleCoil(addr)));
            resp = ResponseAdu {hdr: header, pdu: resp_data}; // copy MBAP header, put data to service request
        },
        _ => ()
    };

    // resp_tcp_buf is the TCP datagram
    let resp_tcp_buf = &mut [0u8; 4096];
    let n = encode_response(resp, resp_tcp_buf); // form a TCP buffer from the response

    // once all is done, call write on the socket and flush

    Ok(())
}

pub fn din_get_cb(pin: usize, hal: &Io) -> bool {
    let di_hdl = &hal.din;
    let res = match di_hdl[pin].get_level() {
        Level::Low => false,
        Level::High => true,
    };
    res
}

pub fn dout_set_cb(pin: usize, hal: &mut Io, val: bool) {
    let dout_hdl = &mut hal.dout;
    let val = match val {
        false => Level::Low,
        true => Level::High
    };
    dout_hdl[pin].set_level(val);
}

pub fn dout_get_cb(pin: usize, hal: &Io) -> bool {
    let di_hdl = &hal.dout;
    let res = match di_hdl[pin].get_output_level() {
        Level::Low => false,
        Level::High => true,
    };
    res
}
