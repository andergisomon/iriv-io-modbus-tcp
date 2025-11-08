use defmt::*;
use embassy_net::tcp::TcpSocket;
use embassy_rp::gpio::{Input, Level};
use modbus_core::tcp::{RequestAdu, ResponseAdu};
use modbus_core::tcp::server::{self, decode_request, encode_response};
use modbus_core::{Error, Data, FunctionCode, Request, RequestPdu, Response};
use crate::{Hal, Io};

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

    let req = modbus_core::tcp::server::decode_request(buf)?.unwrap();

    // handle what kind of modbus request it is
    // just use match statements here to handle client read/writes
    let RequestAdu {hdr: header, pdu: req_data} = req;

    // copy MBAP header, put data to service request
    // resp_data need to be populated by a helper function that relays data from the DI/O and AI
    // just use match statements here to handle client read/writes
    let resp = ResponseAdu {hdr: header, pdu: resp_data};

    // resp_tcp_buf is the TCP datagram
    let n = encode_response(resp, resp_tcp_buf); // form a TCP buffer from the response

    // once all is done, call write on the socket and flush

    Ok(())
}

pub fn din_get_cb(pin: usize, hal: &mut Io) -> u8 {
    let di_hdl = &mut hal.din;
    let res = match di_hdl[pin].get_level() {
        Level::Low => 0,
        Level::High => 1,
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
