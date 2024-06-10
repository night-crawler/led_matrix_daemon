use image::Luma;

pub mod device_version;
pub mod port;

pub const FWK_MAGIC: &[u8] = &[0x32, 0xAC];

const WIDTH: usize = 9;
const HEIGHT: usize = 34;

#[derive(Clone, Copy, Debug, PartialEq, clap::ValueEnum)]
#[repr(u8)]
pub enum Pattern {
    Percentage = 0,
    Gradient = 1,
    DoubleGradient = 2,
    LotusSideways = 3,
    Zigzag = 4,
    AllOn = 5,
    Panic = 6,
    LotusTopDown = 7,
    //AllBrightnesses
}

#[allow(dead_code)]
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

fn pixel_to_brightness(pixel: &Luma<u8>) -> u8 {
    let brightness = pixel.0[0];
    let enhanced_brightness = 255.0 / (1.0 + f64::exp(-0.03 * (brightness as f64 - 128.0)));
    enhanced_brightness as u8
}
