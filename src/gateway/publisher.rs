//!
//! Channel author
//!
use crate::{
    payload::payload_serializer::{empty_bytes, json::PayloadBuilder, PacketPayload},
    random_seed,
};

use anyhow::Result;
use iota_streams::app::transport::tangle::client::Client;
use iota_streams::app::transport::tangle::{client::SendTrytesOptions, PAYLOAD_BYTES};
use iota_streams::app_channels::api::tangle::{Address, Author};
use std::string::ToString;

///
/// Channel
///
pub struct Channel {
    author: Author<Client>,
    channel_address: String,
    announcement_id: String,
    previous_msg_tag: String,
}

impl Channel {
    ///
    /// Initialize the Channel
    ///
    pub fn new(node: String, send_opt: SendTrytesOptions, seed_option: Option<String>) -> Channel {
        let seed = match seed_option {
            Some(seed) => seed,
            None => random_seed(),
        };

        let client: Client = Client::new(
            send_opt,
            iota::client::ClientBuilder::new()
                .node(&node)
                .unwrap()
                .build()
                .unwrap(),
        );

        let author = Author::new(&seed, "utf-8", PAYLOAD_BYTES, false, client);

        let channel_address = author.channel_address().unwrap().to_string();

        Self {
            author: author,
            channel_address: channel_address,
            announcement_id: String::default(),
            previous_msg_tag: String::default(),
        }
    }

    ///
    /// Open a channel
    ///
    pub fn open(&mut self) -> Result<(String, String)> {
        let announcement_message = self.author.send_announce()?;

        self.announcement_id = announcement_message.msgid.to_string();
        println!("{:?}", self.announcement_id);

        Ok((self.channel_address.clone(), self.announcement_id.clone()))
    }

    ///
    /// Write signed packet
    ///
    pub fn write_signed<T>(&mut self, data: T) -> Result<String>
    where
        T: serde::Serialize,
    {
        let payload = PayloadBuilder::new().public(&data).unwrap().build();
        let signed_packet_link = {
            if self.previous_msg_tag == String::default() {
                let keyload_link =
                    Address::from_str(&self.channel_address, &self.announcement_id).unwrap();
                let msg = self.author.send_signed_packet(
                    &keyload_link,
                    &payload.public_data(),
                    &empty_bytes(),
                )?;
                let ret_link = msg.0;
                ret_link.clone()
            } else {
                let msg = self.author.send_signed_packet(
                    &Address::from_str(&self.channel_address, &self.previous_msg_tag).unwrap(),
                    &payload.public_data(),
                    &empty_bytes(),
                )?;
                let ret_link = msg.0;
                ret_link.clone()
            }
        };

        self.previous_msg_tag = signed_packet_link.msgid.to_string().clone();

        Ok(signed_packet_link.msgid.to_string())
    }
}
