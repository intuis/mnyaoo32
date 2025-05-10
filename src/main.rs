use core::str;
use std::io::{Read, Write};
use std::thread;
use std::{error::Error, net::TcpStream};

use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

use embedded_hal::spi::MODE_3;

use esp_idf_svc::hal::gpio::{Gpio16, Gpio18, Gpio19, Gpio23, Gpio5};
use esp_idf_svc::hal::spi::{SpiDriver, SPI2};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay::Ets,
        gpio::{AnyIOPin, PinDriver},
        modem::Modem,
        peripherals::Peripherals,
        spi::{SpiConfig, SpiDeviceDriver, SpiDriverConfig},
        units::*,
    },
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use esp_idf_sys::esp;
use log::info;
use mipidsi::{interface::SpiInterface, models::ST7789, Builder};

use mipidsi::options::{ColorInversion, Orientation};
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig};
use ratatui::prelude::Backend;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Terminal;

const DISPLAY_OFFSET: (u16, u16) = (52, 40);
const DISPLAY_SIZE: (u16, u16) = (135, 240);

struct EmbeddedApp {
    event_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
}

type Display = mipidsi::Display<
    SpiInterface<
        'static,
        SpiDeviceDriver<'static, SpiDriver<'static>>,
        PinDriver<'static, Gpio16, esp_idf_svc::hal::gpio::Output>,
    >,
    ST7789,
    PinDriver<'static, Gpio23, esp_idf_svc::hal::gpio::Output>,
>;

impl EmbeddedApp {
    fn init() -> Result<(Self, Display), Box<dyn Error>> {
        let config = esp_idf_sys::esp_vfs_eventfd_config_t { max_fds: 1 };
        esp! { unsafe { esp_idf_sys::esp_vfs_eventfd_register(&config) } }?;

        let event_loop = EspSystemEventLoop::take()?;
        let nvs = EspDefaultNvsPartition::take()?;
        let peripherals = Peripherals::take()?;
        let wifi = Self::init_wifi(&event_loop, &nvs, peripherals.modem)?;
        std::mem::forget(wifi);

        let mut backlight = PinDriver::output(peripherals.pins.gpio4)?;
        backlight.set_high()?;
        std::mem::forget(backlight);

        let spi_interface = Self::init_spi_interface(
            peripherals.pins.gpio5,
            peripherals.spi2,
            peripherals.pins.gpio18,
            peripherals.pins.gpio19,
            peripherals.pins.gpio16,
        )?;

        let mut delay = Ets;
        let mut display = Builder::new(ST7789, spi_interface)
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(PinDriver::output(peripherals.pins.gpio23)?)
            .display_offset(DISPLAY_OFFSET.0, DISPLAY_OFFSET.1)
            .orientation(Orientation::new().rotate(mipidsi::options::Rotation::Deg90))
            .display_size(DISPLAY_SIZE.0, DISPLAY_SIZE.1)
            .init(&mut delay)
            .expect("Failed to init display");

        // Reset pixels
        display
            .clear(Rgb565::BLACK)
            .expect("Failed to clear display");

        let embedded_app = Self { event_loop, nvs };

        info!("embedded app initialized");

        Ok((embedded_app, display))
    }

    fn init_wifi(
        event_loop: &EspSystemEventLoop,
        nvs: &EspDefaultNvsPartition,
        modem: Modem,
    ) -> Result<BlockingWifi<EspWifi<'static>>, Box<dyn Error>> {
        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, event_loop.clone(), Some(nvs.clone()))?,
            event_loop.clone(),
        )?;

        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: env!("WIFI_SSID").try_into().expect("WIFI_SSID is too long"),
            password: env!("WIFI_PASS")
                .try_into()
                .expect("WIFI_PASSWORD is too long"),
            ..Default::default()
        }))?;
        wifi.start()?;
        wifi.connect()?;
        wifi.wait_netif_up()?;

        info!("wifi initialized");

        Ok(wifi)
    }

    fn init_spi_interface(
        gpio5: Gpio5,
        spi: SPI2,
        gpio18: Gpio18,
        gpio19: Gpio19,
        gpio16: Gpio16,
    ) -> std::result::Result<
        SpiInterface<
            'static,
            esp_idf_svc::hal::spi::SpiDeviceDriver<
                'static,
                esp_idf_svc::hal::spi::SpiDriver<'static>,
            >,
            esp_idf_svc::hal::gpio::PinDriver<
                'static,
                esp_idf_svc::hal::gpio::Gpio16,
                esp_idf_svc::hal::gpio::Output,
            >,
        >,
        Box<dyn Error>,
    > {
        let config = SpiConfig::new()
            .baudrate(80.MHz().into())
            .data_mode(MODE_3)
            .write_only(true);
        let spi_device = SpiDeviceDriver::new_single(
            spi,
            gpio18,
            gpio19,
            Option::<AnyIOPin>::None,
            Some(gpio5),
            &SpiDriverConfig::new(),
            &config,
        )?;
        let buffer = Box::leak(Box::new([0_u8; 512]));
        let spi_interface = SpiInterface::new(spi_device, PinDriver::output(gpio16)?, buffer);

        info!("spi interface initialized");

        Ok(spi_interface)
    }
}

struct MessageToDisplay {
    line: Line<'static>,
    height: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let (_embedded_app, mut display) = EmbeddedApp::init()?;

    let backend = EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let size = backend.size()?;
    let mut terminal = Terminal::new(backend)?;

    let mut stream = TcpStream::connect("irc.libera.chat:6665").unwrap();
    let mut buf = vec![0; 1024];
    let mut messages_to_display: Vec<MessageToDisplay> = vec![];

    info!("Stream initialized");

    authenticate(&mut stream);

    info!("Authenticated to IRC");

    let mut height_available = size.height;
    loop {
        let stream_read = stream.read(&mut buf);
        match stream_read {
            Ok(0) => break,
            Ok(bytes_read) => {
                let raw_msg = str::from_utf8(&buf[..bytes_read]).unwrap().trim_end();
                let owned_msg = raw_msg.to_owned();

                let irc_message = IrcMessage::parse(&owned_msg);

                let msg = match irc_message {
                    IrcMessage::Message {
                        from,
                        channel,
                        content,
                    } => {
                        let msg_len = from.chars().count() + 1 + content.chars().count();
                        let colors = [
                            Color::Blue,
                            Color::Cyan,
                            Color::Green,
                            Color::LightBlue,
                            Color::Magenta,
                            Color::Red,
                            Color::Yellow,
                            Color::LightCyan,
                            Color::LightGreen,
                        ];
                        let mut sum: usize = 0;
                        for byte in from.as_bytes() {
                            sum += *byte as usize;
                        }

                        let color = colors[sum % 9];
                        let spans = vec![
                            Span::styled(from, Style::default().fg(color)),
                            Span::styled(": ", Style::default().fg(Color::Gray)),
                            Span::raw(content),
                        ];
                        Some((msg_len, spans))
                    }
                    IrcMessage::Ping(_) => {
                        stream.write_all(b"PONG\r\n").unwrap();
                        None
                    }
                    IrcMessage::Raw(raw_msg) => {
                        Some((raw_msg.chars().count(), vec![Span::raw(raw_msg.to_owned())]))
                    }
                    IrcMessage::Notice(notice_msg) => {
                        Some((notice_msg.chars().count(), vec![Span::raw(notice_msg)]))
                    }
                    IrcMessage::Join { user, .. } => {
                        // If you want join messages
                        // Some((
                        //     user.chars().count(),
                        //     vec![
                        //         Span::styled("+", Style::default().fg(Color::Green)),
                        //         Span::styled(user, Style::default().gray()),
                        //     ],
                        // ));
                        None
                    }
                    IrcMessage::Quit { user, from } => None,
                    IrcMessage::Part { user, from } => None,
                };
                if let Some((msg_len, msg)) = msg {
                    let width_by_chars = u16::try_from(msg_len).unwrap() / size.width;
                    let lines_will_occupy = width_by_chars + 1;

                    if lines_will_occupy > 5 {
                        info!("Skipping a long message!");
                        continue;
                    }

                    while height_available < lines_will_occupy {
                        info!("Removing a message!");
                        height_available += messages_to_display[0].height;
                        messages_to_display.remove(0);
                    }

                    height_available = height_available.saturating_sub(width_by_chars + 1);

                    let line = Line::from(msg);
                    let msg_to_display = MessageToDisplay {
                        line,
                        height: width_by_chars + 1,
                    };
                    messages_to_display.push(msg_to_display);
                }
            }
            Err(_) => todo!(),
        };
        buf.fill(0);
        draw(&mut terminal, &messages_to_display);
    }

    loop {
        thread::park();
    }
}

enum IrcMessage {
    Message {
        from: String,
        channel: String,
        content: String,
    },
    Notice(String),
    Join {
        user: String,
        to: String,
    },
    Quit {
        user: String,
        from: String,
    },
    Part {
        user: String,
        from: String,
    },
    Ping(String),
    Raw(String),
}

impl IrcMessage {
    fn parse(raw_msg: &str) -> Self {
        if raw_msg.starts_with("PING") {
            Self::Ping(raw_msg[6..].to_string())
        } else {
            let mut split: Vec<_> = raw_msg.split(' ').collect();
            if split.len() >= 3 {
                let name = split[0];
                let parsed_name = Self::parse_name(name);
                let command = split[1];
                let parameter = split[2];
                if command == "PRIVMSG" {
                    if let Some(first_word) = split[3..].get_mut(0) {
                        *first_word = &first_word[1..];
                    };

                    IrcMessage::Message {
                        from: parsed_name.to_owned(),
                        channel: parameter.to_owned(),
                        content: split[3..].join(" "),
                    }
                } else if command == "QUIT" {
                    Self::Quit {
                        user: parsed_name.to_string(),
                        from: parameter.to_string(),
                    }
                } else if command == "PART" {
                    Self::Part {
                        user: parsed_name.to_string(),
                        from: parameter.to_string(),
                    }
                } else if command == "JOIN" {
                    Self::Join {
                        user: parsed_name.to_string(),
                        to: parameter.to_string(),
                    }
                } else if command == "NOTICE" {
                    Self::Notice(split[3..].join(" "))
                } else if command == "PING" {
                    Self::Ping(parameter[1..].to_owned())
                } else {
                    Self::Raw(raw_msg.to_owned())
                }
            } else {
                Self::Raw(raw_msg.to_owned())
            }
        }
    }

    fn parse_name(raw_name: &str) -> &str {
        info!("Parsing name: {raw_name}");
        // Skip colon
        let Some(name) = &raw_name.get(1..) else {
            return raw_name;
        };

        let end_of_name = match name.find('!') {
            Some(idx) => idx,
            None => return name,
        };

        &name[..end_of_name]
    }
}

fn draw<'display, D: DrawTarget<Color = C>, C: PixelColor>(
    terminal: &mut Terminal<EmbeddedBackend<'display, D, C>>,
    messages_to_display: &[MessageToDisplay],
) where
    EmbeddedBackend<'display, D, C>: Backend,
{
    terminal
        .draw(|f| {
            let area = f.area();
            let paragraph = Paragraph::new(Text::from(
                messages_to_display
                    .iter()
                    .map(|msg| msg.line.clone())
                    .collect::<Vec<_>>(),
            ))
            .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);
        })
        .unwrap();
}

fn authenticate(stream: &mut TcpStream) {
    stream.write_all("NICK esp32test\r\n".as_bytes()).unwrap();
    info!("Set nick");
    stream
        .write_all("USER esp32test 0 * :Test\r\n".as_bytes())
        .unwrap();
    stream.write_all("JOIN #linux\r\n".as_bytes()).unwrap();
}
