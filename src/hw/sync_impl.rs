use std::thread;
use std::time::Duration;

use anyhow::bail;
use image::io::Reader as ImageReader;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};

use crate::hw::{Command, FRAMEWORK_VID, FWK_MAGIC, HEIGHT, LED_MATRIX_PID, Pattern, pixel_to_brightness, SERIAL_TIMEOUT, WIDTH};

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
            vec![LED_MATRIX_PID, 0x22, 0xFF]
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
                bail!("Permission denied, couldn't access inputmodule serialport. \
                Ensure that you have permission, for example using a udev rule or sudo.");
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
        let mut brightnesses = [0; HEIGHT];

        for y in 0..HEIGHT {
            let pixel = img.get_pixel(x as u32, y as u32);
            brightnesses[y] = pixel_to_brightness(pixel);
        }

        send_col(&mut port, x as u8, &brightnesses)?;
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

