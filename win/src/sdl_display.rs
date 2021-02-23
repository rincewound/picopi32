use core1::Display;
use sdl2::{video};

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use video::{Window, WindowContext};

struct Context
{
    texure_creator: sdl2::render::TextureCreator<WindowContext>,
    display_target: Canvas<Window> 
}

impl Context
{
    pub fn new(target: sdl2::render::Canvas<video::Window>) -> Self
    {
        let creator = target.texture_creator();
        let result = Self{
            texure_creator: creator,
            display_target: target
        };
        return result;
    }
}

const Total_Num_Color_Elements: usize = 320 * 240*3;

pub struct SdlDisplay
{
    backbuffer: sdl2::render::Texture,
    ctx: Context,
    pixel_index: usize,
    raw_buffer: [u8; Total_Num_Color_Elements]
}

impl SdlDisplay
{
    pub fn new(target: sdl2::render::Canvas<video::Window>) -> Self
    {
        let ctx = Context::new(target);
        let result = SdlDisplay{
            backbuffer: ctx.texure_creator.create_texture_streaming(PixelFormatEnum::RGB24, 320, 240).unwrap(),
            ctx: ctx,
            pixel_index: 0,
            raw_buffer: [0; Total_Num_Color_Elements]
        };
        return result;
    }
}

impl Display for SdlDisplay
{
    fn push_pixel(&mut self, color: core1::Color) {
        let pi = self.pixel_index;
        self.raw_buffer[pi + 0] = color.R;
        self.raw_buffer[pi + 1] = color.G;
        self.raw_buffer[pi + 2] = color.B;
        self.pixel_index += 3;
    }

    fn reset_position(&mut self) {
        self.ctx.display_target.clear();
        self.pixel_index = 0;
    }

    fn show_screen(&mut self) {
        let buff_slice = &self.raw_buffer[0..Total_Num_Color_Elements];
        let _ = self.backbuffer.with_lock(None, |buffer: &mut [u8], _pitch: usize|
            {
                for i in 0..Total_Num_Color_Elements
                {
                    buffer[i] = buff_slice[i];
                }
            });

        let _= self.ctx.display_target.copy_ex(
            &self.backbuffer,
            None,
            Some(Rect::new(0, 0, 320, 240)),
            0.0,
            None,
            false,
            false,
        );
        self.ctx.display_target.present();
        self.pixel_index = 0;
    }
}