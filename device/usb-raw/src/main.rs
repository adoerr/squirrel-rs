#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    peripherals::USB,
    usb::{Driver, InterruptHandler},
};
use embassy_usb::{
    Builder, Config, Handler,
    control::{InResponse, OutResponse, Recipient, Request, RequestType},
    types::InterfaceNumber,
};
#[allow(unused_imports)]
use {defmt_rtt as _, panic_probe as _};

const USB_VENDOR_ID: u16 = 0x4242;
const USB_PRODUCT_ID: u16 = 0x4242;

bind_interrupts!(struct Irqs{
     USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("init raw USB device");

    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);

    let mut cfg = Config::new(USB_VENDOR_ID, USB_PRODUCT_ID);
    cfg.manufacturer = Some("Squirrel");
    cfg.product = Some("USB Squirrel");
    cfg.serial_number = Some("CAFECAFE");
    cfg.max_power = 100;
    cfg.max_packet_size_0 = 64;

    let mut cfg_desc = [0; 64];
    let mut bos_desc = [0; 64];
    let mut msos_desc = [0; 64];
    let mut ctrl_buf = [0; 64];

    let mut ctrl_handler = ControlHandler {
        if_num: InterfaceNumber(0),
    };

    let mut builder = Builder::new(
        driver,
        cfg,
        &mut cfg_desc,
        &mut bos_desc,
        &mut msos_desc,
        &mut ctrl_buf,
    );

    let mut function = builder.function(0xFF, 0, 0);
    let mut interface = function.interface();
    let _alt = interface.alt_setting(0xFF, 0, 0, None);

    ctrl_handler.if_num = interface.interface_number();

    drop(function);

    builder.handler(&mut ctrl_handler);

    let mut usb = builder.build();

    usb.run().await;
}

/// Handle CONTROL endpoint requests and responses. For many simple requests and responses
/// you can get away with only using the control endpoint.
struct ControlHandler {
    if_num: InterfaceNumber,
}

impl Handler for ControlHandler {
    /// Respond to HostToDevice control messages, where the host sends us a command and
    /// optionally some data, and we can only acknowledge or reject it.
    fn control_out<'a>(&'a mut self, req: Request, buf: &'a [u8]) -> Option<OutResponse> {
        // Log the request before filtering to help with debugging.
        info!("got control_out, request={}, buf={:a}", req, buf);

        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor || req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        // Accept request 100, value 200, reject others.
        if req.request == 100 && req.value == 200 {
            Some(OutResponse::Accepted)
        } else {
            Some(OutResponse::Rejected)
        }
    }

    /// Respond to DeviceToHost control messages, where the host requests some data from us.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        info!("got control_in, request={}", req);

        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor || req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        // Respond "hello" to request 101, value 201, when asked for 5 bytes, otherwise reject.
        if req.request == 101 && req.value == 201 && req.length == 5 {
            buf[..5].copy_from_slice(b"hello");
            Some(InResponse::Accepted(&buf[..5]))
        } else {
            Some(InResponse::Rejected)
        }
    }
}
