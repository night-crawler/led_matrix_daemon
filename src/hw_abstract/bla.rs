use anyhow::bail;
use std::thread;
use std::time::Duration;

use image::{io::Reader as ImageReader, Luma};
use serialport::{SerialPort, SerialPortInfo, SerialPortType};

use crate::hw_abstract::led_matrix::Pattern;

const FWK_MAGIC: &[u8] = &[0x32, 0xAC];
pub const FRAMEWORK_VID: u16 = 0x32AC;
pub const LED_MATRIX_PID: u16 = 0x0020;
pub const B1_LCD_PID: u16 = 0x0021;

type Brightness = u8;

// TODO: Use a shared enum with the firmware code
#[derive(Clone, Copy)]
#[repr(u8)]
enum Command {
    Brightness = 0x00,
    Pattern = 0x01,
    Bootloader = 0x02,
    Sleeping = 0x03,
    Animate = 0x04,
    Panic = 0x05,
    DisplayBwImage = 0x06,
    SendCol = 0x07,
    CommitCols = 0x08,
    _B1Reserved = 0x09,
    StartGame = 0x10,
    GameControl = 0x11,
    _GameStatus = 0x12,
    SetColor = 0x13,
    DisplayOn = 0x14,
    InvertScreen = 0x15,
    SetPixelColumn = 0x16,
    FlushFramebuffer = 0x17,
    ClearRam = 0x18,
    ScreenSaver = 0x19,
    Fps = 0x1A,
    PowerMode = 0x1B,
    AnimationPeriod = 0x1C,
    PwmFreq = 0x1E,
    DebugMode = 0x1F,
    Version = 0x20,
}

enum GameControlArg {
    _Up = 0,
    _Down = 1,
    _Left = 2,
    _Right = 3,
    Exit = 4,
    _SecondLeft = 5,
    _SecondRight = 6,
}

const WIDTH: usize = 9;
const HEIGHT: usize = 34;

const SERIAL_TIMEOUT: Duration = Duration::from_millis(2000);

fn match_serialdevs(
    ports: &[SerialPortInfo],
    requested: &Option<String>,
    pid: Option<u16>,
) -> Vec<String> {
    if let Some(requested) = requested {
        for p in ports {
            if requested == &p.port_name {
                return vec![p.port_name.clone()];
            }
        }
        vec![]
    } else {
        let mut compatible_devs = vec![];
        let pids = if let Some(pid) = pid {
            vec![pid]
        } else {
            // By default, accept any type
            vec![LED_MATRIX_PID, B1_LCD_PID, 0x22, 0xFF]
        };
        // Find all supported Framework devices
        for p in ports {
            if let SerialPortType::UsbPort(usbinfo) = &p.port_type {
                if usbinfo.vid == FRAMEWORK_VID && pids.contains(&usbinfo.pid) {
                    compatible_devs.push(p.port_name.clone());
                }
            }
        }
        compatible_devs
    }
}

// pub fn find_serialdevs(args: &crate::ClapCli, wait_for_device: bool) -> (Vec<String>, bool) {
//     let mut serialdevs: Vec<String>;
//     let mut waited = false;
//     loop {
//         let ports = serialport::available_ports().expect("No ports found!");
//         if args.list || args.verbose {
//             for p in &ports {
//                 match &p.port_type {
//                     SerialPortType::UsbPort(usbinfo) => {
//                         println!("{}", p.port_name);
//                         println!("  VID     {:#06X}", usbinfo.vid);
//                         println!("  PID     {:#06X}", usbinfo.pid);
//                         if let Some(sn) = &usbinfo.serial_number {
//                             println!("  SN      {}", sn);
//                         }
//                         if let Some(product) = &usbinfo.product {
//                             // TODO: Seems to replace the spaces with underscore, not sure why
//                             println!("  Product {}", product);
//                         }
//                     }
//                     _ => {
//                         //println!("{}", p.port_name);
//                         //println!("  Unknown (PCI Port)");
//                     }
//                 }
//             }
//         }
//         serialdevs = match_serialdevs(
//             &ports,
//             &args.serial_dev,
//             args.command.as_ref().map(|x| x.to_pid()),
//         );
//         if serialdevs.is_empty() {
//             if wait_for_device {
//                 // Waited at least once, that means the device was not present
//                 // when the program started
//                 waited = true;
//
//                 // Try again after short wait
//                 thread::sleep(Duration::from_millis(100));
//                 continue;
//             } else {
//                 return (vec![], waited);
//             }
//         } else {
//             break;
//         }
//     }
//     (serialdevs, waited)
// }

fn get_device_version(serialdev: &str) -> anyhow::Result<()> {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    simple_cmd_port(&mut port, Command::Version, &[])?;

    let mut response: Vec<u8> = vec![0; 32];
    port.read_exact(response.as_mut_slice())
        .expect("Found no data!");

    let major = response[0];
    let minor = (response[1] & 0xF0) >> 4;
    let patch = response[1] & 0x0F;
    let pre_release = response[2] == 1;
    print!("Device Version: {major}.{minor}.{patch}");
    if pre_release {
        print!(" (Pre-Release)");
    }
    println!();

    Ok(())
}

fn pattern_cmd(serialdev: &str, arg: Pattern) -> anyhow::Result<()> {
    simple_cmd(serialdev, Command::Pattern, &[arg as u8])
}

fn simple_cmd_multiple(
    serialdevs: &Vec<String>,
    command: Command,
    args: &[u8],
) -> anyhow::Result<()> {
    for serialdev in serialdevs {
        simple_cmd(serialdev, command, args)?;
    }
    Ok(())
}

fn simple_cmd(serialdev: &str, command: Command, args: &[u8]) -> anyhow::Result<()> {
    let port_result = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open();

    match port_result {
        Ok(mut port) => simple_cmd_port(&mut port, command, args)?,
        Err(error) => match error.kind {
            serialport::ErrorKind::Io(std::io::ErrorKind::PermissionDenied) => {
                bail!("Permission denied, couldn't access inputmodule serialport. Ensure that you have permission, for example using a udev rule or sudo.");
            }
            other_error => {
                bail!("Couldn't open port: {:?}", other_error);
            }
        },
    }
    Ok(())
}

fn open_serialport(serialdev: &str) -> Box<dyn SerialPort> {
    serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port")
}

fn simple_open_cmd(
    serialport: &mut Box<dyn SerialPort>,
    command: Command,
    args: &[u8],
) -> anyhow::Result<()> {
    simple_cmd_port(serialport, command, args)
}

fn simple_cmd_port(
    port: &mut Box<dyn SerialPort>,
    command: Command,
    args: &[u8],
) -> anyhow::Result<()> {
    let mut buffer: [u8; 64] = [0; 64];
    buffer[..2].copy_from_slice(FWK_MAGIC);
    buffer[2] = command as u8;
    buffer[3..3 + args.len()].copy_from_slice(args);
    port.write_all(&buffer[..3 + args.len()])?;

    Ok(())
}

fn sleeping_cmd(serialdev: &str, arg: Option<bool>) -> anyhow::Result<()> {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(goto_sleep) = arg {
        simple_cmd_port(&mut port, Command::Sleeping, &[u8::from(goto_sleep)])?;
    } else {
        simple_cmd_port(&mut port, Command::Sleeping, &[])?;

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let sleeping: bool = response[0] == 1;
        println!("Currently sleeping: {sleeping}");
    }

    Ok(())
}

fn brightness_cmd(serialdev: &str, arg: Option<u8>) -> anyhow::Result<()> {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(brightness) = arg {
        simple_cmd_port(&mut port, Command::Brightness, &[brightness])?;
    } else {
        simple_cmd_port(&mut port, Command::Brightness, &[])?;

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let brightness: u8 = response[0];
        println!("Current brightness: {brightness}");
    }

    Ok(())
}

fn animate_cmd(serialdev: &str, arg: Option<bool>) -> anyhow::Result<()>{
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(animate) = arg {
        simple_cmd_port(&mut port, Command::Animate, &[animate as u8])?;
    } else {
        simple_cmd_port(&mut port, Command::Animate, &[])?;

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let animating = response[0] == 1;
        println!("Currently animating: {animating}");
    }
    
    Ok(())
}

/// Stage greyscale values for a single column. Must be committed with commit_cols()
fn send_col(port: &mut Box<dyn SerialPort>, x: u8, vals: &[u8]) -> anyhow::Result<()> {
    let mut buffer: [u8; 64] = [0; 64];
    buffer[0] = x;
    buffer[1..vals.len() + 1].copy_from_slice(vals);
    simple_cmd_port(port, Command::SendCol, &buffer[0..vals.len() + 1])
}

/// Commit the changes from sending individual cols with send_col(), displaying the matrix.
/// This makes sure that the matrix isn't partially updated.
fn commit_cols(port: &mut Box<dyn SerialPort>) -> anyhow::Result<()> {
    simple_cmd_port(port, Command::CommitCols, &[])
}

///Increase the brightness with each pixel.
///Only 0-255 available, so it can't fill all 306 LEDs
fn all_brightnesses_cmd(serialdev: &str) -> anyhow::Result<()> {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    for x in 0..WIDTH {
        let mut vals: [u8; HEIGHT] = [0; HEIGHT];

        for y in 0..HEIGHT {
            let brightness = x + WIDTH * y;
            vals[y] = if brightness > 255 { 0 } else { brightness } as u8;
        }

        send_col(&mut port, x as u8, &vals)?;
    }
    commit_cols(&mut port)?;

    Ok(())
}

fn breathing_cmd(serialdevs: &Vec<String>) -> anyhow::Result<()> {
    loop {
        // Go quickly from 250 to 50
        for i in 0..40 {
            simple_cmd_multiple(serialdevs, Command::Brightness, &[250 - i * 5])?;
            thread::sleep(Duration::from_millis(25));
        }

        // Go slowly from 50 to 0
        for i in 0..50 {
            simple_cmd_multiple(serialdevs, Command::Brightness, &[50 - i])?;
            thread::sleep(Duration::from_millis(10));
        }

        // Go slowly from 0 to 50
        for i in 0..50 {
            simple_cmd_multiple(serialdevs, Command::Brightness, &[i])?;
            thread::sleep(Duration::from_millis(10));
        }

        // Go quickly from 50 to 250
        for i in 0..40 {
            simple_cmd_multiple(serialdevs, Command::Brightness, &[50 + i * 5])?;
            thread::sleep(Duration::from_millis(25));
        }
    }
}

// // Calculate pixel brightness from an RGB triple
// fn pixel_to_brightness(pixel: &Luma<u8>) -> u8 {
//     let brightness = pixel.0[0];
//     // Poor man's scaling to make the greyscale pop better.
//     // Should find a good function.
//     if brightness > 200 {
//         brightness
//     } else if brightness > 150 {
//         ((brightness as u32) * 10 / 8) as u8
//     } else if brightness > 100 {
//         brightness / 2
//     } else if brightness > 50 {
//         brightness
//     } else {
//         brightness * 2
//     }
// }


// fn pixel_to_brightness(pixel: &Luma<u8>) -> u8 {
//     let brightness = pixel.0[0];
//     // Apply a linear transformation to enhance contrast
//     let enhanced_brightness = if brightness > 200 {
//         brightness
//     } else if brightness > 150 {
//         ((brightness as u32) * 12 / 10) as u8 // Adjust scaling factor as needed
//     } else if brightness > 100 {
//         ((brightness as u32) * 10 / 10) as u8
//     } else if brightness > 50 {
//         ((brightness as u32) * 8 / 10) as u8
//     } else {
//         ((brightness as u32) * 6 / 10) as u8
//     };
//     enhanced_brightness
// }

fn pixel_to_brightness(pixel: &Luma<u8>) -> u8 {
    let brightness = pixel.0[0];
    // Enhance contrast using a sigmoid function
    let enhanced_brightness = 255.0 / (1.0 + f64::exp(-0.03 * (brightness as f64 - 128.0)));
    enhanced_brightness as u8
}


/// Display an image in greyscale
/// Sends each 1x34 column and then commits => 10 commands
pub fn display_gray_image_cmd(serialdev: &str, image_path: &str) -> anyhow::Result<()> {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()?;

    let img = ImageReader::open(image_path)?.decode()?.to_luma8();

    let width = img.width();
    let height = img.height();

    if width != WIDTH as u32 || height != HEIGHT as u32 {
        bail!("Image must be 9x34 pixels");
    }

    for x in 0..WIDTH {
        let mut vals: [u8; HEIGHT] = [0; HEIGHT];

        for y in 0..HEIGHT {
            let pixel = img.get_pixel(x as u32, y as u32);
            vals[y] = pixel_to_brightness(pixel);
        }

        send_col(&mut port, x as u8, &vals)?;
    }
    commit_cols(&mut port)?;

    Ok(())
}

/// Show a black/white matrix
/// Send everything in a single command
fn render_matrix(serialdev: &str, matrix: &[[u8; 34]; 9]) -> anyhow::Result<()> {
    // One bit for each LED, on or off
    // 39 = ceil(34 * 9 / 8)
    let mut vals: [u8; 39] = [0x00; 39];

    for x in 0..9 {
        for y in 0..34 {
            let i = x + 9 * y;
            if matrix[x][y] == 0xFF {
                vals[i / 8] |= 1 << (i % 8);
            }
        }
    }

    simple_cmd(serialdev, Command::DisplayBwImage, &vals)?;
    
    Ok(())
}

/// Render up to five 5x6 pixel font items
fn show_font(serialdev: &str, font_items: &[Vec<u8>]) -> anyhow::Result<()>{
    let mut vals: [u8; 39] = [0x00; 39];

    for (digit_i, digit_pixels) in font_items.iter().enumerate() {
        let offset = digit_i * 7;
        for pixel_x in 0..5 {
            for pixel_y in 0..6 {
                let pixel_value = digit_pixels[pixel_x + pixel_y * 5];
                let i = (2 + pixel_x) + (9 * (pixel_y + offset));
                if pixel_value == 1 {
                    vals[i / 8] |= 1 << (i % 8);
                }
            }
        }
    }

    simple_cmd(serialdev, Command::DisplayBwImage, &vals)?;
    
    Ok(())
}

fn animation_fps_cmd(serialdev: &str, arg: Option<u16>) -> anyhow::Result<()> {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()
        .expect("Failed to open port");

    if let Some(fps) = arg {
        let period = (1000 / fps).to_le_bytes();
        simple_cmd_port(&mut port, Command::AnimationPeriod, &[period[0], period[1]])?;
    } else {
        simple_cmd_port(&mut port, Command::AnimationPeriod, &[])?;

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())?;

        let period = u16::from_le_bytes([response[0], response[1]]);
        println!("Animation Frequency: {}ms / {}Hz", period, 1_000 / period);
    }

    Ok(())
}

fn pwm_freq_cmd(serialdev: &str, arg: Option<u16>) -> anyhow::Result<()> {
    let mut port = serialport::new(serialdev, 115_200)
        .timeout(SERIAL_TIMEOUT)
        .open()?;

    if let Some(freq) = arg {
        let hz = match freq {
            29000 => 0,
            3600 => 1,
            1800 => 2,
            900 => 3,
            _ => panic!("Invalid frequency"),
        };
        simple_cmd_port(&mut port, Command::PwmFreq, &[hz])?;
    } else {
        simple_cmd_port(&mut port, Command::PwmFreq, &[])?;

        let mut response: Vec<u8> = vec![0; 32];
        port.read_exact(response.as_mut_slice())
            .expect("Found no data!");

        let hz = match response[0] {
            0 => 29000,
            1 => 3600,
            2 => 1800,
            3 => 900,
            _ => panic!("Invalid frequency"),
        };
        println!("Animation Frequency: {}Hz", hz);
    }

    Ok(())
}
