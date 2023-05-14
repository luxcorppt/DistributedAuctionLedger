use std::sync::Arc;
use log::{debug, info, warn};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::rpc::data::{SignedKadMessage};
use crate::{MESSAGE_BUFFER_LENGTH, Result};
use crate::node::NodeInfo;

pub(crate) async fn wait_message_recv(socket: Arc<UdpSocket>, dispatch: Sender<SignedKadMessage>) {
    let mut buf = [0;MESSAGE_BUFFER_LENGTH];
    
    info!("Start socket listener at {:?}", socket.local_addr());
    loop {
        let (len, _addr) = socket.recv_from(&mut buf).await.unwrap();
        let msg = String::from_utf8_lossy(&buf[0..len]);
        let msg = match SignedKadMessage::from_string(&msg) {
            Ok(msg) => {
                debug!("Received message from {} -> {:?}", _addr, msg);
                msg
            }
            Err(e) => {
                warn!("Got an error when parsing received message. Ignoring.\nError: {:?}", e);
                continue;
            }
        };
        // TODO: This will crash and burn if too many messages. Make a handler or begin dropping them.
        dispatch.try_send(msg).unwrap()
    }
    // Ok(())
}

pub(crate) async fn send_message(socket: &UdpSocket, message: &SignedKadMessage, target: &NodeInfo) -> Result<()> {
    let buffer = message.as_message()?;
    let addr = target.socket_addr();
    debug!("Sending message to {} -> {:?}", addr, message);
    let _ = socket.send_to(buffer.as_ref(), addr).await?;
    Ok(())
}
