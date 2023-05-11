use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::rpc::data::KadMessage;
use crate::{MESSAGE_BUFFER_LENGTH, Result};
use crate::node::NodeInfo;

pub(crate) async fn wait_message_recv(socket: Arc<UdpSocket>, dispatch: Sender<KadMessage>) {
    let mut buf = [0;MESSAGE_BUFFER_LENGTH];

    loop {
        let (len, _addr) = socket.recv_from(&mut buf).await.unwrap();
        let msg = String::from_utf8_lossy(&buf[0..len]);
        let msg = match KadMessage::from_string(&msg) {
            Ok(msg) => {msg}
            Err(e) => {
                eprintln!("Got an error when parsing received message. Ignoring.\nError: {:?}", e);
                continue;
            }
        };
        // TODO: This will crash and burn if too many messages. Make a handler or begin dropping them.
        dispatch.try_send(msg).unwrap()
    }

    // Ok(())
}

pub(crate) async fn send_message(socket: &UdpSocket, message: &KadMessage, target: &NodeInfo) -> Result<()> {
    let buffer = message.as_message()?;
    let addr = target.socket_addr();
    let _ = socket.send_to(buffer.as_ref(), addr).await?;
    Ok(())
}
