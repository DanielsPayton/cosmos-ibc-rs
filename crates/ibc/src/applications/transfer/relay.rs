//! Implements the processing logic for ICS20 (token transfer) message.

use super::context::{TokenTransferExecutionContext, TokenTransferValidationContext};
use crate::applications::transfer::error::TokenTransferError;
use crate::applications::transfer::is_sender_chain_source;
use crate::applications::transfer::packet::PacketData;
use crate::core::ics04_channel::packet::Packet;
use crate::prelude::*;

pub mod on_recv_packet;
pub mod send_transfer;

pub fn refund_packet_token_execute<D>(
    ctx_a: &mut impl TokenTransferExecutionContext<D>,
    packet: &Packet,
    data: &PacketData,
) -> Result<(), TokenTransferError> {
    let sender = data
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        packet.port_id_on_a.clone(),
        packet.chan_id_on_a.clone(),
        &data.token.denom,
    ) {
        ctx_a.unescrow_coins_execute(
            &packet.port_id_on_a,
            &packet.chan_id_on_a,
            &sender,
            &data.token,
        )
    }
    // mint vouchers back to sender
    else {
        ctx_a.mint_coins_execute(&sender, &data.token)
    }
}

pub fn refund_packet_token_validate<D>(
    ctx_a: &impl TokenTransferValidationContext<D>,
    packet: &Packet,
    data: &PacketData,
) -> Result<(), TokenTransferError> {
    let sender = data
        .sender
        .clone()
        .try_into()
        .map_err(|_| TokenTransferError::ParseAccountFailure)?;

    if is_sender_chain_source(
        packet.port_id_on_a.clone(),
        packet.chan_id_on_a.clone(),
        &data.token.denom,
    ) {
        ctx_a.unescrow_coins_validate(
            &packet.port_id_on_a,
            &packet.chan_id_on_a,
            &sender,
            &data.token,
        )
    } else {
        ctx_a.mint_coins_validate(&sender, &data.token)
    }
}
