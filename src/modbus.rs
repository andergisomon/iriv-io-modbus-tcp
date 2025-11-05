use embassy_net::tcp::TcpSocket;
use modbus_core::tcp::ResponseAdu;
use modbus_core::tcp::server::{self, decode_request, encode_response};
use modbus_core::{Data, FunctionCode, Request, RequestPdu, Response};

pub async fn transact<'r>(buf: &mut [u8], socket: TcpSocket<'static>, req: RequestPdu<'r>) -> Result<Response<'r>> {

    let mut total_read = 0;

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
        info!("rxd {}", core::str::from_utf8(&buf[..n]).unwrap());
    }


    if total_read == 0 {
        // Timeout if nothing has been received
        Err(Error::Timeout)
    } else {
        let data = &self.buffer[..total_read];
        debug!("Received: ({}) {:x}", total_read, data);

        // Try to parse the response
        let response = modbus_core::rtu::client::decode_response(data)
            .map_err(|_| Error::Transport)?
            .ok_or(Error::Modbus)?;

        Ok(response.pdu.0.map_err(|_| Error::Modbus)?)
    }



    let resp = ResponseAdu {
        hdr: Header {

        }
    }

    let n = encode_response(adu, buf)
}

pub async fn get_client_req_pdu() {

}
