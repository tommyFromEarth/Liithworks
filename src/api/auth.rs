use napi_derive::napi;

#[napi]
pub mod auth {
    use std::net::SocketAddr;

    use napi::bindgen_prelude::{BigInt, Buffer, Error};
    use steamworks::{
        networking_types::NetworkingIdentity, AuthSessionTicketResponse, AuthTicket, SteamId,
        TicketForWebApiResponse,
    };
    use tokio::sync::oneshot;
    use log::{info, error, debug}; // Asegúrate de tener el crate log configurado para usar logs.

    #[napi]
    pub struct Ticket {
        pub(crate) data: Vec<u8>,
        pub(crate) handle: AuthTicket,
    }

    #[napi]
    impl Ticket {
        #[napi]
        pub fn cancel(&mut self) {
            let client = crate::client::get_client();
            client.user().cancel_authentication_ticket(self.handle);
        }

        #[napi]
        pub fn get_bytes(&self) -> Buffer {
            self.data.clone().into()
        }
    }

    #[napi]
    pub async fn get_session_ticket_with_steam_id(
        steam_id64: BigInt,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        get_session_ticket(
            NetworkingIdentity::new_steam_id(SteamId::from_raw(steam_id64.get_u64().1)),
            timeout_seconds,
        )
        .await
    }

    #[napi]
    pub async fn get_session_ticket_with_ip(
        ip: String,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        match ip.parse::<SocketAddr>() {
            Ok(addr) => get_session_ticket(NetworkingIdentity::new_ip(addr), timeout_seconds).await,
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    async fn get_session_ticket(
        network_identity: NetworkingIdentity,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);

        let (ticket_handle, ticket) = client
            .user()
            .authentication_session_ticket(network_identity);

        info!("Requesting authentication session ticket...");
        debug!("Ticket handle created: {:?}", ticket_handle);
        debug!("Initial ticket data: {:?}", ticket);

        let callback =
            client.register_callback(move |session_ticket_response: AuthSessionTicketResponse| {
                if session_ticket_response.ticket == ticket_handle {
                    info!("Received session ticket response: {:?}", session_ticket_response);
                    if let Some(tx) = tx.take() {
                        tx.send(match session_ticket_response.result {
                            Ok(()) => Ok(()),
                            Err(e) => {
                                error!("Error in session ticket response: {}", e);
                                Err(Error::from_reason(e.to_string()))
                            },
                        })
                        .unwrap();
                    }
                }
            });

        let mut ticket = Ticket {
            data: ticket,
            handle: ticket_handle,
        };

        let timeout_seconds = u64::from(timeout_seconds.unwrap_or(10));
        let result =
            tokio::time::timeout(std::time::Duration::from_secs(timeout_seconds), rx).await;

        drop(callback);

        match result {
            Ok(result) => match result {
                Ok(Ok(())) => {
                    info!("Session ticket successfully received.");
                    debug!("Final ticket data: {:?}", ticket.data);
                    Ok(ticket)
                },
                Ok(Err(e)) => {
                    ticket.cancel();
                    error!("Session ticket error: {}", e);
                    Err(e)
                }
                Err(e) => {
                    ticket.cancel();
                    error!("Error receiving session ticket: {}", e);
                    Err(Error::from_reason(e.to_string()))
                }
            },
            Err(_) => {
                ticket.cancel();
                error!("Steam didn't validate the ticket in time.");
                Err(Error::from_reason(
                    "Steam didn't validate the ticket in time.",
                ))
            }
        }
    }

    #[napi]
    pub async fn get_auth_ticket_for_web_api(
        identity: String,
        timeout_seconds: Option<u32>,
    ) -> Result<Ticket, Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
        let mut tx = Some(tx);
    
        let ticket_handle = client
            .user()
            .authentication_session_ticket_for_webapi(&identity);
    
        info!("Requesting authentication session ticket for Web API...");
        debug!("Ticket handle created for Web API: {:?}", ticket_handle);
    
        let callback =
            client.register_callback(move |ticket_for_webapi_response: TicketForWebApiResponse| {
                if ticket_for_webapi_response.ticket_handle == ticket_handle {
                    let ticket = ticket_for_webapi_response.ticket.clone(); // Clona el ticket aquí
                    info!("Received ticket for Web API response: {:?}", ticket_for_webapi_response);
                    if let Some(tx) = tx.take() {
                        tx.send(match ticket_for_webapi_response.result {
                            Ok(()) => Ok(ticket),
                            Err(e) => {
                                error!("Error in Web API ticket response: {}", e);
                                Err(Error::from_reason(e.to_string()))
                            },
                        })
                        .unwrap();
                    }
                }
            });
    
        let timeout_seconds = u64::from(timeout_seconds.unwrap_or(10));
        let result =
            tokio::time::timeout(std::time::Duration::from_secs(timeout_seconds), rx).await;
    
        drop(callback);
    
        match result {
            Ok(result) => match result {
                Ok(Ok(data)) => {
                    info!("Web API session ticket successfully received.");
                    debug!("Final ticket data for Web API: {:?}", data);
                    Ok(Ticket {
                        handle: ticket_handle,
                        data,
                    })
                },
                Ok(Err(e)) => {
                    client.user().cancel_authentication_ticket(ticket_handle);
                    error!("Web API session ticket error: {}", e);
                    Err(e)
                }
                Err(e) => {
                    client.user().cancel_authentication_ticket(ticket_handle);
                    error!("Error receiving Web API session ticket: {}", e);
                    Err(Error::from_reason(e.to_string()))
                }
            },
            Err(_) => {
                client.user().cancel_authentication_ticket(ticket_handle);
                error!("Steam didn't validate the Web API ticket in time.");
                Err(Error::from_reason(
                    "Steam didn't validate the Web API ticket in time.",
                ))
            }
        }
    }
}    