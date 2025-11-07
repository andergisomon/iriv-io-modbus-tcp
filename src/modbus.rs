use defmt::*;
use embassy_net::tcp::TcpSocket;
use modbus_core::tcp::{RequestAdu, ResponseAdu};
use modbus_core::tcp::server::{self, decode_request, encode_response};
use modbus_core::{Data, FunctionCode, Request, RequestPdu, Response};

#[derive(PartialEq, Eq, Debug, Format, Clone)]
pub enum Error {
    Modbus,
    Transport,
    Timeout,
}

/// Services client reads and writes
/// TCP socket timeout is set in main()
pub async fn transact_client<'r>(buf: &mut [u8], socket: TcpSocket<'static>) -> Result<Response<'r>> {

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

    let req = modbus_core::tcp::server::decode_request(buf)
        .map_err(|_| Error::Transport)?
        .ok_or(Error::Modbus)?;

    // handle what kind of modbus request it is
    // just use match statements here to handle client read/writes
    let RequestAdu {hdr: header, pdu: req_data} = req;

    // copy MBAP header, put data to service request
    // resp_data need to be populated by a helper function that relays data from the DI/O and AI
    // just use match statements here to handle client read/writes
    let resp = ResponseAdu {hdr: header, pdu: resp_data};

    // resp_tcp_buf is the TCP datagram
    let n = encode_response(resp, resp_tcp_buf) // form a TCP buffer from the response
}
