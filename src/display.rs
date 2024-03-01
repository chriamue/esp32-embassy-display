// source: https://github.com/andy31415/rust-esp32-c3-demos/blob/oled_wifi/src/main.rs
// https://randomnerdtutorials.com/esp32-ssd1306-oled-display-arduino-ide/
// https://www.roboter-bausatz.de/p/lora-esp32-entwicklungsboard-sx1278-mit-0.96-oled-display-v3-868mhz-915mhz
use anyhow::Result;
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii::FONT_4X6, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{InputPin, Output, OutputPin, Pin, PinDriver},
    i2c::{I2c, I2cConfig, I2cDriver},
    peripheral::Peripheral,
    prelude::FromValueType,
};
use image::{imageops, DynamicImage, ImageBuffer, Luma};
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::I2CInterface,
    rotation::DisplayRotation,
    size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};

pub struct Display<'a, T: OutputPin>
where
    T: OutputPin,
{
    display: Ssd1306<
        I2CInterface<I2cDriver<'a>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,
    reset: PinDriver<'a, T, Output>,
}

pub type DisplayType = Ssd1306<
    I2CInterface<I2cDriver<'static>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

impl<'a, T: Pin> Display<'a, T>
where
    T: OutputPin,
{
    pub fn new<I2C: I2c>(
        i2c: impl Peripheral<P = I2C> + 'a,
        rst: impl Peripheral<P = T> + 'a,
        sda: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        scl: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
    ) -> Result<Display<'a, T>> {
        // lifetime is important here
        // or the display will turn off
        let reset: PinDriver<'a, T, Output> = PinDriver::output(rst).unwrap();

        let config = I2cConfig::new().baudrate(400.kHz().into());

        let i2c_driver: I2cDriver<'a> = I2cDriver::new(i2c, sda, scl, &config).unwrap();
        let interface: I2CInterface<I2cDriver<'a>> = I2CDisplayInterface::new(i2c_driver);

        let display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        Ok(Self { display, reset })
    }

    pub fn init(&mut self) -> Result<()> {
        self.reset.set_high().unwrap();
        self.display
            .init()
            .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.reset.set_high().unwrap();
        FreeRtos::delay_ms(1);
        self.reset.set_low().unwrap();
        FreeRtos::delay_ms(10);
        self.reset.set_high().unwrap();
        self.init()?;
        Ok(())
    }

    pub fn print(&mut self, text: &str) -> Result<()> {
        self.display.clear_buffer();
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_4X6)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline(text, Point::new(0, 16), text_style, Baseline::Top)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Txt2 error: {:?}", e))?;

        self.display
            .flush()
            .map_err(|e| anyhow::anyhow!("Flush error: {:?}", e))?;

        Ok(())
    }

    pub fn show(&mut self, image: ImageRaw<BinaryColor>) -> Result<()> {
        self.display.clear_buffer();
        let im = Image::new(&image, Point::new(0, 0));
        im.draw(&mut self.display).unwrap();
        self.display.flush().unwrap();
        Ok(())
    }

    pub fn show_dynamic_image(&mut self, dynamic_image: DynamicImage) -> Result<()> {
        let (binary_data, width, height) = convert_dynamic_image_to_binary_raw(dynamic_image);
        let image_raw = ImageRaw::<BinaryColor>::new(&binary_data, width);

        self.display.clear_buffer();
        let im = Image::new(&image_raw, Point::zero());
        im.draw(&mut self.display).expect("Failed to draw image");

        self.display.flush().expect("Failed to flush display");

        Ok(())
    }
}

fn convert_dynamic_image_to_binary_raw(image: DynamicImage) -> (Vec<u8>, u32, u32) {
    // Target dimensions
    let target_width = image.width() + 8 - (image.width() % 8);
    let target_height = image.height() + 8 - (image.height() % 8);

    // Convert the input image to grayscale
    let mut gray_image = image.to_luma8();

    // Resize or pad the image to fit 128x64 dimensions
    let (width, height) = gray_image.dimensions();
    if width != target_width || height != target_height {
        // If the image is smaller than the target in either dimension, pad it
        if width < target_width || height < target_height {
            let x_padding = (target_width - width) / 2;
            let y_padding = (target_height - height) / 2;
            // Create a new image with the target dimensions filled with black (0)
            let mut padded_image = ImageBuffer::from_pixel(target_width, target_height, Luma([0]));
            // Copy the original image into the center of the padded image
            imageops::overlay(
                &mut padded_image,
                &gray_image,
                x_padding.into(),
                y_padding.into(),
            );
            gray_image = padded_image;
        } else {
            // If the image is larger, resize it (this might distort the image)
            gray_image = imageops::resize(
                &gray_image,
                target_width,
                target_height,
                imageops::FilterType::Nearest,
            );
        }
    }

    // Now, convert the adjusted image to binary format
    let buffer_size = (target_width as usize * target_height as usize + 7) / 8;
    let mut binary_buffer = vec![0u8; buffer_size];

    for (x, y, pixel) in gray_image.enumerate_pixels() {
        let luma = pixel.0[0];
        let index = x as usize + y as usize * target_width as usize; // Adjust index calculation for target dimensions
        let byte_index = index / 8;
        let bit_index = 7 - (index % 8);

        // Set the bit if the pixel is closer to white
        if luma > 128 {
            binary_buffer[byte_index] |= 1 << bit_index;
        }
    }

    (binary_buffer, target_width, target_height)
}
