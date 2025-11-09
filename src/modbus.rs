use defmt::*;
use embassy_net::tcp::TcpSocket;
use embassy_rp::gpio::{Level};
use modbus_core::tcp::{Header, RequestAdu, ResponseAdu};
use modbus_core::tcp::server::{decode_request, encode_response};
use modbus_core::{Coils, Data, Error, Request, RequestPdu, Response, ResponsePdu};
use crate::{Io, an_read_current_ua, an_read_voltage_mv};

#[derive(Debug)]
pub enum CallbackError {
    /// Error when client specifies an undefined register address.
    NoAddressMatch,
    /// Error when client sends a Modbus request that is not yet implemented.
    NoSupportedRequestMatch,
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
const MAX_NUMBER_OF_CONTACTS: usize = 11;

/// Input Registers (3x) - Read-only
const ANV0_ADDR: u16 = 0x0200; /// Analog Input 0 (V)
const ANV1_ADDR: u16 = 0x0201; /// Analog Input 1 (V)
const ANA0_ADDR: u16 = 0x0210; /// Analog Input 0 (mA)
const ANA1_ADDR: u16 = 0x0211; /// Analog Input 1 (mA)
const MAX_NUMBER_OF_INPUT_REGISTERS: usize = 5;

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
pub async fn transact_client(buf: &mut [u8], hal: &mut Io, socket: &mut TcpSocket<'_>) -> Result<(), Error> {
    // read one modbus request
    let n = match socket.read(buf).await {
        Ok(0) => {
            warn!("read EOF");
            return Ok(());
        }
        Ok(n) => n,
        Err(e) => {
            warn!("read error: {:?}", e);
            return Ok(());
        }
    };

    let req = match decode_request(buf) {
        Ok(Some(r)) => r,
        Ok(None) => {
            warn!("Incomplete Modbus request");
            return Ok(());
        }
        Err(e) => {
            warn!("Failed to decode request: {:?}", e);
            return Ok(());
        }
    };
    let RequestAdu {hdr: header, pdu: req_data} = req;

    let resp_data;
    let mut resp: Result<ResponseAdu<'_>, CallbackError> = Err(CallbackError::NoSupportedRequestMatch);
    let target_buf = &mut [0u8; MAX_NUMBER_OF_COILS];

    match req_data {
        // Read Digital Outputs
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
            resp = Ok(ResponseAdu {hdr: header, pdu: resp_data}); // copy MBAP header, put data to service request
        },
        // Read Digital Inputs
        RequestPdu(Request::ReadDiscreteInputs(addr, quantity)) => {
            let bools = &mut [false; MAX_NUMBER_OF_CONTACTS];
            for i in 0..quantity as usize {
                bools[i] = din_get_cb(i, &hal);
            }

            let start_addr: Result<usize, CallbackError> = match addr {
                DI0_ADDR => Ok(0),
                DI1_ADDR => Ok(1),
                DI2_ADDR => Ok(2),
                DI3_ADDR => Ok(3),
                DI4_ADDR => Ok(4),
                DI5_ADDR => Ok(5),
                DI6_ADDR => Ok(6),
                DI7_ADDR => Ok(7),
                DI8_ADDR => Ok(8),
                DI9_ADDR => Ok(9),
                DI10_ADDR => Ok(10),
                _ => {
                    error!("Modbus address given does not match any register definition");
                    Err(CallbackError::NoAddressMatch)
                }
            };

            let truncated_bools = &mut bools[start_addr.unwrap_or_else(|_| -> usize {0})..];
            let coils = Coils::from_bools(truncated_bools, target_buf).unwrap();
            // modbus-core should've really distinguished between coils and contacts
            resp_data = ResponsePdu(Ok(Response::ReadDiscreteInputs(coils)));
            resp = Ok(ResponseAdu {hdr: header, pdu: resp_data}); // copy MBAP header, put data to service request
        },
        // Read Analog Inputs, Hardware and Firmware Info
        RequestPdu(Request::ReadInputRegisters(addr, quantity)) => {
            let words = &mut [0u16; MAX_NUMBER_OF_INPUT_REGISTERS];

            // Modbus Request for Analog Current Input
            if (addr == ANA0_ADDR) || (addr == ANA1_ADDR) {
                for i in 0..quantity as usize {
                    words[i] = an_read_current_ua(hal, i).await as u16;
                }
            }
            // Modbus Request for Analog Voltage Input
            if (addr == ANV0_ADDR) || (addr == ANV1_ADDR) {
                for i in 0..quantity as usize {
                    words[i] = an_read_voltage_mv(hal, i).await as u16;
                }
            }
            // Modbus Request for Miscellaneous Info
            let read_misc
                =  (addr == MODEL1_ADDR)
                || (addr == MODEL2_ADDR)
                || (addr == VERSION_MAJOR_ADDR)
                || (addr == VERSION_MINOR_ADDR)
                || (addr == VERSION_PATCH_ADDR);
            let misc_info = [MODEL1_VAL, MODEL2_VAL, VERSION_MAJOR_VAL, VERSION_MINOR_VAL, VERSION_PATCH_VAL];

            if read_misc {
                for i in 0..quantity as usize {
                    words[i] = misc_info[i];
                }
            }

            let start_addr: Result<usize, CallbackError> = match addr {
                ANA0_ADDR => Ok(0),
                ANA1_ADDR => Ok(1),
                ANV0_ADDR => Ok(0),
                ANV1_ADDR => Ok(1),
                MODEL1_ADDR         => Ok(0),
                MODEL2_ADDR         => Ok(1),
                VERSION_MAJOR_ADDR  => Ok(2),
                VERSION_MINOR_ADDR  => Ok(3),
                VERSION_PATCH_ADDR  => Ok(4),
                _ => {
                    error!("Modbus address given does not match any register definition");
                    Err(CallbackError::NoAddressMatch)
                }
            };

            let truncated_words = &mut words[start_addr.unwrap_or_else(|_| -> usize {0})..];
            resp_data = ResponsePdu(Ok(Response::ReadInputRegisters(Data::from_words(truncated_words, target_buf).unwrap())));
            resp = Ok(ResponseAdu {hdr: header, pdu: resp_data}); // copy MBAP header, put data to service request
        },
        // Write to One Specific Digital Output
        RequestPdu(Request::WriteSingleCoil(addr, val)) => {
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

            match start_addr {
                Ok(pin) => {
                    dout_set_cb(pin, hal, val);
                    resp_data = ResponsePdu(Ok(Response::WriteSingleCoil(addr as u16)));
                    resp = Ok(ResponseAdu {hdr: header, pdu: resp_data}); // copy MBAP header, put data to service request
                },
                _ => ()
            }
        },
        _ => {
            error!("Client sent unimplemented Modbus request");
            resp = Err(CallbackError::NoSupportedRequestMatch)
        }
    };

    // resp_tcp_buf is the TCP datagram
    let resp_tcp_buf = &mut [0u8; 64];
    match resp {
        Ok(resp) => {
            _ = encode_response(resp, resp_tcp_buf); // form a TCP buffer from the response
            // once all is done, here call write on the socket and flush
            _ = socket.write(resp_tcp_buf).await.map_err(|_| -> () {});
        }
        _ => error!("Client sent unimplemented Modbus request")
    }

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
