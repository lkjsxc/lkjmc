mod auth;
mod command;
pub(crate) mod routes;
mod server;

pub(crate) use server::serve;
