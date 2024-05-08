use log::{error, info};
use ssh2::{Session, Stream};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::Duration;

use crate::common;
use crate::common::command;
use crate::common::exception::{Exception, Result};

#[derive(Clone)]
pub struct RemoteClient {
    pub host: String,
    pub client: Session,
}

impl Default for RemoteClient {
    fn default() -> Self {
        todo!()
    }
}

impl Debug for RemoteClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl RemoteClient {
    pub async fn get_available_remote_client(
        user: &str,
        password: &str,
        host: String,
        port: i32,
    ) -> Self {
        info!("try connect to {host}");
        let mut timeout = 5;
        loop {
            let client = Self::new(user, password, host.as_str(), port);
            if client.is_err() {
                info!("wait 5 seconds to get {host} remote client");
                tokio::time::sleep(Duration::from_secs(5)).await;
                timeout += 5;
            } else {
                info!("connect {host} success!");
                return client.unwrap();
            }
            if timeout >= 600 {
                let e = format!("{timeout} seconds timeout to connect remote {host}",);
                error!("{e}");
                panic!("{e}");
            }
        }
    }
    pub fn new(user: &str, password: &str, host: &str, port: i32) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        match connect(addr.as_str(), user, password) {
            Ok(v) => Ok(Self {
                host: host.to_string(),
                client: v,
            }),
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }

    pub fn deploy_ustack(&self, command: &str) {
        info!("start deploy");
        match self.exec_long_command(command, false) {
            Ok(_) => {
                info!("deploy success");
            }
            Err(e) => {
                error!("deploy failed");
                error!("error is {e}");
            }
        };
    }

    pub fn exec_command(&self, command: &str) -> Result<String> {
        exec(&self.client, command)
    }
    pub fn create_image(&self, format: &str, path: &str, name: &str) -> String {
        let command = common::command::create_image(format, path, name);
        match exec(&self.client, command.as_str()) {
            Ok(v) => String::from(v.trim()),
            Err(e) => {
                error!("error is {}", e);
                panic!("create image error")
            }
        }
    }

    pub fn exec_long_command(&self, command: &str, show_error: bool) -> Result<()> {
        info!("exec '{command}'");
        let mut channel = self.client.channel_session()?;
        channel.exec(command)?;
        let stream: Stream;
        if show_error {
            stream = channel.stderr();
        } else {
            stream = channel.stream(0);
        }
        let _ = thread::scope(|s| {
            s.spawn(|| {
                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    info!("{}", line.unwrap());
                }
            });
        });
        let code = channel.exit_status()?;
        if code != 0 && !show_error {
            let mut stream = channel.stderr();
            let mut error = String::new();
            stream.read_to_string(&mut error)?;
            channel.close()?;
            channel.wait_close()?;
            let error = format!("exec '{}' error \n {}", command, error.trim());
            error!("{error}");
            return Err(Exception::Error(format!(
                "exec '{command}' failed\n {}",
                error.trim()
            )));
        }
        channel.close()?;
        channel.wait_close()?;
        Ok(())
    }

    pub fn upload(&self, file: &str, remote_file: &str) {
        info!("upload {file} to {}", self.host);
        match upload(&self.client, file, remote_file) {
            Ok(_) => {
                info!("upload {file} to {} success", self.host);
            }
            Err(e) => {
                error!("{}", e);
                panic!("{}", e);
            }
        };
    }
    pub fn get_nics(&self) -> (String, String) {
        let result = self.exec_command(command::get_nics());
        match result {
            Ok(v) => {
                let nics: Vec<String> = v.trim().split("\n").map(|x| x.to_string()).collect();
                if nics.len() >= 2 {
                    (nics[0].to_string(), nics[1].to_string())
                } else {
                    (nics[0].to_string(), nics[0].to_string())
                }
            }
            Err(e) => {
                error!("{}", e);
                panic!("{}", e);
            }
        }
    }

    pub fn set_hostname(&self, host: &str) {
        info!("set {} hostname to {host}", self.host);
        let _ = self.exec_command(command::set_hostname(host.to_string()).as_str());
    }
}

fn connect(addr: &str, user: &str, password: &str) -> Result<Session> {
    let tcp = TcpStream::connect(addr)?;
    let mut session = Session::new()?;
    session.set_tcp_stream(tcp);
    session.handshake()?;
    session.userauth_password(user, password)?;
    Ok(session)
}

fn exec(session: &Session, command: &str) -> Result<String> {
    info!("exec '{command}'");
    let mut channel = session.channel_session()?;
    channel.exec(command)?;
    let mut result = String::new();
    channel.read_to_string(&mut result)?;
    let code = channel.exit_status()?;
    info!("exit code is {code}");
    if code != 0 {
        let mut stream = channel.stderr();
        let mut error = String::new();
        stream.read_to_string(&mut error)?;
        channel.close()?;
        channel.wait_close()?;
        let error = format!("exec '{}' error \n {}", command, error.trim());
        error!("{error}");
        Err(Exception::Error(format!(
            "exec '{command}' failed\n {}",
            error.trim()
        )))
    } else {
        channel.close()?;
        channel.wait_close()?;
        Ok(result)
    }
}

pub fn upload(session: &Session, file: &str, remote_file: &str) -> Result<()> {
    let path = Path::new(file);
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let size = path.metadata()?.len();
    let mut channel = session.scp_send(Path::new(remote_file), 0o644, size, None)?;
    channel.write_all(data.as_bytes())?;
    channel.send_eof()?;
    channel.wait_eof()?;
    channel.close()?;
    channel.wait_close()?;
    Ok(())
}
