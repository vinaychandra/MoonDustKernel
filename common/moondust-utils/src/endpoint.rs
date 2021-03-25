use alloc::collections::LinkedList;
use alloc::sync::Arc;

use crate::sync::{mutex::Mutex, once::AsyncOnce};

enum EndpointItem<TRequest, TResponse> {
    ServerWaiting(AsyncOnce<(TRequest, AsyncOnce<TResponse>)>),
    ClientWaiting(TRequest, AsyncOnce<TResponse>),
}

pub struct Endpoint<TRequest, TResponse> {
    endpoints: Mutex<LinkedList<EndpointItem<TRequest, TResponse>>>,
}

impl<TRequest, TResponse> Endpoint<TRequest, TResponse> {
    pub const fn new() -> Self {
        Self {
            endpoints: Mutex::new(LinkedList::new()),
        }
    }

    pub async fn wait_for_request(&self) -> (TRequest, AsyncOnce<TResponse>) {
        let mut endpoints = self.endpoints.lock().await;
        match endpoints.pop_front() {
            Some(item) => {
                match item {
                    EndpointItem::ServerWaiting(s) => {
                        // Another server is waiting. Push the retrieved and the current one to the list
                        endpoints.push_front(EndpointItem::ServerWaiting(s));

                        let client_request;
                        {
                            let ecs = AsyncOnce::new();
                            endpoints.push_back(EndpointItem::ServerWaiting(ecs.clone()));
                            core::mem::drop(endpoints);
                            client_request = ecs.await;
                        }
                        match Arc::try_unwrap(client_request) {
                            Ok(val) => val,
                            Err(_) => panic!("Expected that the Arc has only one reference"),
                        }
                    }
                    EndpointItem::ClientWaiting(a, b) => {
                        // A client is waiting for us. Return it.
                        (a, b)
                    }
                }
            }
            None => {
                // No endpoints. Just push this
                let client_request;
                {
                    let ecs = AsyncOnce::new();
                    endpoints.push_back(EndpointItem::ServerWaiting(ecs.clone()));
                    core::mem::drop(endpoints);
                    client_request = ecs.await;
                }
                match Arc::try_unwrap(client_request) {
                    Ok(val) => val,
                    Err(_) => panic!("Expected that the Arc has only one reference"),
                }
            }
        }
    }

    pub async fn wait_for_response(&self, request: TRequest) -> TResponse {
        let mut endpoints = self.endpoints.lock().await;
        let client_response_async = match endpoints.pop_front() {
            Some(item) => match item {
                EndpointItem::ServerWaiting(s) => {
                    let response_from_server = AsyncOnce::new();
                    s.try_set_result((request, response_from_server.clone()));
                    response_from_server
                }
                EndpointItem::ClientWaiting(a, b) => {
                    // A client is waiting. Push both
                    endpoints.push_front(EndpointItem::ClientWaiting(a, b));
                    let response = AsyncOnce::new();
                    let client_request = EndpointItem::ClientWaiting(request, response.clone());
                    endpoints.push_back(client_request);
                    response
                }
            },
            None => {
                // No servers. Just push this
                let response = AsyncOnce::new();
                let client_request = EndpointItem::ClientWaiting(request, response.clone());
                endpoints.push_back(client_request);
                response
            }
        };

        core::mem::drop(endpoints);
        let client_response = client_response_async.await;

        match Arc::try_unwrap(client_response) {
            Ok(val) => val,
            Err(_) => panic!("Expected that the Arc has only one reference"),
        }
    }
}
