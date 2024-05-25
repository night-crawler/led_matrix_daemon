use std::sync::Arc;
use std::time::Duration;

use anyhow::bail;
use image::{GrayImage, io::Reader as ImageReader};
use serialport::SerialPort;

use crate::config::port_dto::PortDto;
use crate::hw::{Command, FWK_MAGIC, HEIGHT, pixel_to_brightness, WIDTH};
use crate::hw::device_version::DeviceVersion;

#[derive(Debug)]
pub struct Port {
    path: Arc<str>,
    baud_rate: u32,
    timeout: Duration,
    port: Option<Box<dyn SerialPort>>,
    keep_open: bool,
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
        })
    }
}

impl Port {
    fn open(&mut self) -> anyhow::Result<&mut Box<dyn SerialPort>> {
        if self.port.is_none() {
            self.port = Some(
                serialport::new(self.path.as_ref(), self.baud_rate)
                    .timeout(self.timeout)
                    .open()?,
            );
        }

        Ok(self.port.as_mut().unwrap())
    }

    fn close(&mut self) {
        self.port.take();
    }
    fn write_command(&mut self, command: Command, args: &[u8]) -> anyhow::Result<()> {
        let mut buffer: [u8; 64] = [0; 64];
        buffer[..2].copy_from_slice(FWK_MAGIC);
        buffer[2] = command as u8;
        buffer[3..3 + args.len()].copy_from_slice(args);

        self.open()?.write_all(&buffer[..3 + args.len()])?;

        Ok(())
    }

    fn get_device_version(&mut self) -> anyhow::Result<DeviceVersion> {
        let mut response: Vec<u8> = vec![0; 32];

        self.write_command(Command::Version, &[])?;

        let port = self.open()?;
        port.read_exact(response.as_mut_slice())?;

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


    pub fn display_gray_image_by_path(&mut self, image_path: &str) -> anyhow::Result<()> {
        let img = ImageReader::open(image_path)?.decode()?.to_luma8();
        self.display_gray_image(&img)
    }

    pub fn display_gray_image(&mut self, img: &GrayImage) -> anyhow::Result<()> {
        let width = img.width();
        let height = img.height();

        if width != WIDTH as u32 || height != HEIGHT as u32 {
            bail!("Image must be {WIDTH}x{HEIGHT} pixels; got {width}x{height}");
        }

        let mut brightnesses = [0; HEIGHT];

        for col in 0..WIDTH {
            for row in 0..HEIGHT {
                let pixel = img.get_pixel(col as u32, row as u32);
                brightnesses[row] = pixel_to_brightness(pixel);
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
            keep_open: false,
        }
    }

    #[test]
    fn test_get_device_version() {
        assert!(get_port().get_device_version().is_ok());
    }

    #[test]
    fn test_send_col() {
        let mut port = get_port();
        assert!(port.send_col(0, &[0, 10, 2, 3, 4, 5, 6, 70]).is_ok());
        assert!(port.commit_cols().is_ok());
    }

    #[test]
    fn test_display_gray_image() {
        let mut port = get_port();
        assert!(port.display_gray_image_by_path("test_data/img.png").is_ok());
        assert!(port.display_gray_image_by_path("test_data/img.png").is_ok());
    }
}
