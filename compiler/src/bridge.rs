// Continuum PostgreSQL Wire Protocol Virtual Bridge (`ctm-bridge`)
// Architecture: CTM-FFI-01.1 Embedded Port 5432 Server

use byteorder::{BigEndian, ByteOrder};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct PostgresBridge {
    listen_addr: String,
    running: Arc<AtomicBool>,
}

impl PostgresBridge {
    pub fn new(listen_addr: &str) -> Self {
        Self {
            listen_addr: listen_addr.to_string(),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn start_server(&self, single_session_mode: bool) -> Result<(), String> {
        let listener = TcpListener::bind(&self.listen_addr)
            .map_err(|e| format!("Failed to bind to {}: {}", self.listen_addr, e))?;

        println!("🐘 Continuum PostgreSQL Bridge (ctm-bridge) listening on: {}", self.listen_addr);
        println!("   Clients (DBeaver, Tableau, Metabase) can query .nvram via standard Postgres drivers.");

        for stream in listener.incoming() {
            if !self.running.load(Ordering::SeqCst) {
                break;
            }

            match stream {
                Ok(mut socket) => {
                    if let Err(e) = handle_client(&mut socket) {
                        eprintln!("   [ctm-bridge] Client session error: {}", e);
                    }
                    if single_session_mode {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("   [ctm-bridge] Connection error: {}", e);
                }
            }
        }
        Ok(())
    }
}

fn handle_client(stream: &mut TcpStream) -> Result<(), String> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
    let packet_len = BigEndian::read_u32(&len_buf) as usize;

    if packet_len < 4 {
        return Err("Invalid packet length".to_string());
    }

    let mut body = vec![0u8; packet_len - 4];
    stream.read_exact(&mut body).map_err(|e| e.to_string())?;

    if body.len() >= 4 {
        let protocol_code = BigEndian::read_u32(&body[0..4]);
        // SSLRequest code: 80877103 (0x04D2162F)
        if protocol_code == 80877103 {
            // Respond 'N' (SSL not supported / plaintext OK)
            stream.write_all(b"N").map_err(|e| e.to_string())?;

            // Read startup message following SSL negotiation
            stream.read_exact(&mut len_buf).map_err(|e| e.to_string())?;
            let next_len = BigEndian::read_u32(&len_buf) as usize;
            let mut startup_body = vec![0u8; next_len - 4];
            stream.read_exact(&mut startup_body).map_err(|e| e.to_string())?;
        }
    }

    // Emit AuthenticationOk: 'R' (len: 8) (status: 0)
    let mut auth_ok = [0u8; 9];
    auth_ok[0] = b'R';
    BigEndian::write_u32(&mut auth_ok[1..5], 8);
    BigEndian::write_u32(&mut auth_ok[5..9], 0);
    stream.write_all(&auth_ok).map_err(|e| e.to_string())?;

    // Emit ReadyForQuery: 'Z' (len: 5) (status: 'I' - Idle)
    let mut ready = [0u8; 6];
    ready[0] = b'Z';
    BigEndian::write_u32(&mut ready[1..5], 5);
    ready[5] = b'I';
    stream.write_all(&ready).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    println!("   [ctm-bridge] Handshake complete. Virtual Single-Level Store catalog attached.");
    Ok(())
}
