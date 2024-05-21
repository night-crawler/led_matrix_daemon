use crate::hw_abstract::bla::display_gray_image_cmd;

mod hw_abstract;

fn main() -> anyhow::Result<()> {
    let path = "/dev/ttyACM0";

    display_gray_image_cmd(path, "./img.png")?;
    
    Ok(())
}
