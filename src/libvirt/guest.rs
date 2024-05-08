use virt::connect::ConnectCredential;
use virt::connect::{Connect, ConnectAuth};
use virt::domain::Domain;
use virt::error::Error;
use virt::sys;

use crate::libvirt::config::{Guest};
use crate::libvirt::device::{Devices, VideoDeviceModel};
use crate::libvirt::enumerations::DiskDeviceTypes;

pub fn create_domain() -> Result<(), Error> {
    fn callback(creds: &mut Vec<ConnectCredential>) {
        for cred in creds {
            match cred.typed as u32 {
                sys::VIR_CRED_AUTHNAME => {
                    cred.result = Some(String::from("nova"));
                }
                sys::VIR_CRED_PASSPHRASE => {
                    cred.result = Some(String::from("1kLBLpdYIbdYAyEFDpBrYerdMrIEVWdIMOx0oTJC"));
                }
                _ => {
                    panic!("Should not be here...");
                }
            }
        }
    }
    let mut auth = ConnectAuth::new(
        vec![sys::VIR_CRED_AUTHNAME, sys::VIR_CRED_PASSPHRASE],
        callback,
    );
    let mut connection = Connect::open_auth("qemu+tcp://10.10.15.13/system", &mut auth, 0)?;
    let name = String::from("wkt-test");
    let mut guest = Guest::new(name.to_owned(), 4, 10240.0);
    guest.add_device(Devices::new_raw_disk(
        DiskDeviceTypes::Disk,
        "/var/lib/libvirt/images/uos-server-20.20240411.ustack.0-live.x86_64.iso".to_string(),
        "vda".to_string(), "1".to_string()));
    guest.add_device(Devices::new_raw_disk(
        DiskDeviceTypes::Disk,
        "/var/lib/libvirt/images/wkt-test.qcow2".to_string(),
        "vdb".to_string(), "2".to_string()));
    guest.add_device(Devices::new_serial());
    guest.add_device(Devices::Graphics {
        graphics_type: "vnc".to_string(),
        port: "-1".to_string(),
        listen: "0.0.0.0".to_string(),
    });
    guest.add_device(Devices::Video { model: VideoDeviceModel { model_type: "virtio".to_string() } });
    let xml = guest.to_xml();
    println!("xml is {}", xml);
    println!("create instance {}", name);
    let domain = Domain::define_xml(&connection, xml.as_str())?;
    let result = domain.create()?;
    println!("start instance {} {}", name, result);
    // let _ = domain.undefine()?;
    // println!("delete instance {}", name);
    connection.close()?;
    Ok(())
}