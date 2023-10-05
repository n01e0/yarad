use tokio::net::{
    UnixListener,
    UnixStream,
    TcpListener,
    TcpStream,
};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::fs::Permissions;
use crate::config::{Config, StreamType};
use crate::error::*;
use crate::protocol::*;
use log::info;

const BUFFER_SIZE: usize = 4096;

#[derive(Debug)]
pub enum Listener {
    Unix(UnixListener),
    Tcp(TcpListener),
}

impl Listener {
    pub async fn new(config: &Config) -> Result<Listener> {
        Ok(
            match *config.get_stream_type() {
                StreamType::Unix => {
                    let socket_path = config.get_local_socket();
                    if Path::new(socket_path).exists() {
                        std::fs::remove_file(socket_path)?;
                    }
                    let listener = Listener::Unix(UnixListener::bind(socket_path)?);
                    tokio::fs::set_permissions(socket_path, Permissions::from_mode(*config.get_local_socket_mode())).await?;
                    info!("Listening on {}, perm {:#o}", socket_path, *config.get_local_socket_mode());
                    listener

                },
                StreamType::Tcp => {
                    Listener::Tcp(TcpListener::bind(("0.0.0.0", *config.get_tcp_port())).await?)
                },
            }
        )
    }
    pub async fn accept(&self) -> Result<Stream> {
        match self {
            Listener::Unix(listener) => {
                let (stream, _) = listener.accept().await?;
                Ok(Stream::Unix(stream))
            },
            Listener::Tcp(listener) => {
                let (stream, _) = listener.accept().await?;
                Ok(Stream::Tcp(stream))
            },
        }
    }
}

#[derive(Debug)]
pub enum Stream {
    Unix(UnixStream),
    Tcp(TcpStream),
}

impl Stream {
    pub async fn readable(&self) -> Result<()> {
        match self {
            Self::Unix(s) => s.readable().await.map_err(|e| e.into()),
            Self::Tcp(s) => s.readable().await.map_err(|e| e.into()),
        }
    }

    pub async fn writable(&self) -> Result<()> {
        match self {
            Self::Unix(s) => s.writable().await.map_err(|e| e.into()),
            Self::Tcp(s) => s.writable().await.map_err(|e| e.into()),
        }
    }

    pub fn try_parse_commands(&self) -> Result<Vec<Result<Command>>> {
        let raw = self.try_read_to_end()?;
        parse_commands(raw)
    }

    pub fn try_read_to_end(&self) -> Result<Vec<u8>> {
        match self {
            Self::Unix(s) => {
                let mut buf = vec![0; BUFFER_SIZE];
                let mut n = s.try_read(&mut buf[..])?;
                let mut size = n;
                while n == BUFFER_SIZE {
                    let mut tmp = vec![0; BUFFER_SIZE];
                    n = s.try_read(&mut tmp[..])?;
                    buf.extend_from_slice(&tmp[..n]);
                    tmp.clear();
                    size += n;
                }
                buf.truncate(size);
                Ok(buf)
            },
            Self::Tcp(s) => {
                let mut buf = vec![0; BUFFER_SIZE];
                let mut n = s.try_read(&mut buf[..])?;
                let mut size = n;
                while n == BUFFER_SIZE {
                    let mut tmp = vec![0; BUFFER_SIZE];
                    n = s.try_read(&mut tmp[..])?;
                    buf.extend_from_slice(&tmp[..n]);
                    tmp.clear();
                    size += n;
                }
                buf.truncate(size);
                Ok(buf)
            },
        }
    }

    pub fn try_write(&self, buf: &[u8]) -> Result<usize> {
        match self {
            Self::Unix(s) => s.try_write(buf).map_err(|e| e.into()),
            Self::Tcp(s) => s.try_write(buf).map_err(|e| e.into()),
        }
    }
}
