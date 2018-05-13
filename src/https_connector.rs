// use std::io;
// use std::sync::Arc;
//
// use tokio::prelude::*;
// use tokio::net::UdpSocket as TokioUdpSocket;
// use tokio::net::TcpStream;
// use tokio::reactor::Handle;
// use tokio::runtime::Runtime;
//
// use native_tls::TlsConnector;
// use tokio_tls::{TlsConnectorExt, TlsStream};
// use tokio_service::Service;
//
// use hyper::client::HttpConnector;
// use hyper::{Client, Request, Method, Uri};
// pub struct HttpsConnector {
//     tls: Arc<TlsConnector>,
//     http: HttpConnector,
// }
//
// impl Service for HttpsConnector {
//     type Request = Uri;
//     type Response = TlsStream<TcpStream>;
//     type Error = io::Error;
//     type Future = Box<Future<Item = Self::Response, Error = io::Error>>;
//
//     fn call(&self, uri: Uri) -> Self::Future {
//         // Right now this is intended to showcase `https`, but you could
//         // also adapt this to return something like `MaybeTls<T>` where
//         // some clients resolve to TLS streams (https) and others resolve
//         // to normal TCP streams (http)
//         if uri.scheme_part().map(|s| s.as_str()) != Some("https") {
//             // return Box::new(Err(io::Error::new(io::ErrorKind::Other,
//             //                                    "only works with https")))
//         }
//
//         // Look up the host that we're connecting to as we're going to validate
//         // this as part of the TLS handshake.
//         let host: String = match uri.host() {
//             Some(s) => s.to_string(),
//             None =>  {
//                 return Box::new(Future<Item = Self::Response, Error = io::Error>>::new())
//             }
//         };
//
//         // Delegate to the standard `HttpConnector` type to create a connected
//         // TCP socket. Once we've got that socket initiate the TLS handshake
//         // with the host name that's provided in the URI we extracted above.
//         let tls_cx = self.tls.clone();
//         Box::new(self.http.connect(uri).and_then(move |tcp| {
//             tls_cx.connect_async(&host, tcp)
//                 .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
//         }))
//     }
// }
