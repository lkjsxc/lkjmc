mod auth;
mod command;
pub(crate) mod peer;
pub(crate) mod routes;
mod server;

pub(crate) use server::serve;
