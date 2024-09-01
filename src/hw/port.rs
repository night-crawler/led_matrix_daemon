use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use image::{io::Reader as ImageReader, GrayImage, Luma};
use serialport::SerialPort;
use tracing::warn;

use crate::config::port_dto::PortDto;
use crate::hw::device_version::DeviceVersion;
use crate::hw::{Command, FWK_MAGIC, HEIGHT, WIDTH};

#[derive(Debug)]
pub struct Port {
    path: Arc<str>,
    baud_rate: u32,
    timeout: Duration,
    port: Option<Box<dyn SerialPort>>,
    keep_open: bool,
    wait_delay: Option<Duration>,
}

impl TryFrom<PortDto> for Port {
    type Error = anyhow::Error;

    fn try_from(value: PortDto) -> Result<Self, Self::Error> {
        let path: Arc<str> = Arc::from(value.path);

        Ok(Port {
            path,
            port: None,
            baud_rate: value.baud_rate,
            timeout: value.timeout,
            keep_open: value.keep_open,
            wait_delay: value.wait_delay,
        })
    }
}

impl Port {
    fn open(&mut self) -> serialport::Result<&mut Box<dyn SerialPort>> {
        let port = &mut self.port;
        if let Some(port) = port {
            return Ok(port);
        }

        loop {
            let error = match serialport::new(self.path.as_ref(), self.baud_rate)
                .timeout(self.timeout)
                .open()
            {
                Ok(next_port) => {
                    port.replace(next_port);
                    return Ok(port.as_mut().unwrap());
                }
                Err(err) => err,
            };

            if let Some(delay) = self.wait_delay {
                warn!(?error, port = %self.path.as_ref(), "Failed to open port");
                std::thread::sleep(delay);
            } else {
                return Err(error);
            }
        }
    }

    fn close(&mut self) {
        self.port.take();
    }

    fn prepare_command_buffer(&mut self, command: Command, args: &[u8]) -> [u8; 64] {
        let mut buffer: [u8; 64] = [0; 64];
        buffer[..2].copy_from_slice(FWK_MAGIC);
        buffer[2] = command as u8;
        buffer[3..3 + args.len()].copy_from_slice(args);

        buffer
    }
    fn write_command(&mut self, command: Command, data: &[u8]) -> anyhow::Result<()> {
        let buffer = self.prepare_command_buffer(command, data);
        self.open()?.write_all(&buffer[..3 + data.len()])?;
        if !self.keep_open {
            self.close();
        }
        Ok(())
    }

    fn write_read_command(
        &mut self,
        command: Command,
        data: &[u8],
        read_buffer: &mut [u8],
    ) -> anyhow::Result<()> {
        let buffer = self.prepare_command_buffer(command, data);
        let port = self.open()?;

        port.write_all(&buffer[..3 + data.len()])?;
        port.read_exact(read_buffer)?;

        if !self.keep_open {
            self.close();
        }

        Ok(())
    }

    pub fn get_device_version(&mut self) -> anyhow::Result<DeviceVersion> {
        let mut response: Vec<u8> = vec![0; 32];

        self.write_read_command(Command::Version, &[], response.as_mut_slice())?;

        let major = response[0];
        let minor = (response[1] & 0xF0) >> 4;
        let patch = response[1] & 0x0F;
        let pre_release = response[2] == 1;

        Ok(DeviceVersion {
            major,
            minor,
            patch,
            pre_release,
        })
    }

    fn send_col(&mut self, index: u8, vals: &[u8]) -> anyhow::Result<()> {
        let mut buffer: [u8; 64] = [0; 64];
        buffer[0] = index;
        buffer[1..vals.len() + 1].copy_from_slice(vals);
        self.write_command(Command::SendCol, &buffer[0..vals.len() + 1])
    }

    fn commit_cols(&mut self) -> anyhow::Result<()> {
        self.write_command(Command::CommitCols, &[])
    }

    #[allow(dead_code)]
    pub fn display_gray_image_by_path(&mut self, image_path: &str) -> anyhow::Result<()> {
        let img = ImageReader::open(image_path)?.decode()?.to_luma8();
        self.display_gray_image(img)
    }

    pub fn display_gray_image(&mut self, img: GrayImage) -> anyhow::Result<()> {
        let width = img.width();
        let height = img.height();

        if width != WIDTH as u32 || height != HEIGHT as u32 {
            bail!("Image must be {WIDTH}x{HEIGHT} pixels; got {width}x{height}");
        }

        let mut brightnesses = [0; HEIGHT];
        for col in 0..WIDTH {
            for (row, brightness) in brightnesses.iter_mut().enumerate() {
                let &Luma([pixel]) = img.get_pixel(col as u32, row as u32);
                *brightness = pixel;
            }
            self.send_col(col as u8, &brightnesses)?;
        }
        self.commit_cols()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_port() -> Port {
        Port {
            path: Arc::from("/dev/ttyACM0"),
            baud_rate: 115200,
            timeout: Duration::from_secs(20),
            port: None,
            keep_open: true,
            wait_delay: Some(Duration::from_millis(10)),
        }
    }

    #[test]
    fn test_get_device_version() {
        let version = get_port().get_device_version();
        println!("{version:?}");
        assert!(version.is_ok());
    }

    #[test]
    fn test_send_col() {
        let mut port = get_port();
        assert!(port.send_col(1, &[0, 255, 2, 255, 4, 5, 6, 70]).is_ok());
        assert!(port.commit_cols().is_ok());
    }

    #[test]
    fn test_display_gray_image() {
        let mut port = get_port();
        assert!(port
            .display_gray_image_by_path("test_data/img0.png")
            .is_ok());
        assert!(port
            .display_gray_image_by_path("test_data/img0.jpg")
            .is_ok());
    }
}
