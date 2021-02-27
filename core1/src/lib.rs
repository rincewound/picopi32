#![no_std]

pub struct Color
{
    pub r: u8,
    pub g: u8,
    pub b: u8
}

pub enum Irq
{
    Scanline{scanline_index: usize},        // The Scanline IRQ is triggered, when the GfxCore has finished drawing a scanline
    VSync,                                  // The VSync IRQ is triggered, when the GfxCore has finished drawing a complete image
    DmaComplete
}

pub trait Display
{
    fn push_pixel(&mut self, color: Color);
    fn reset_position(&mut self);
    fn show_screen(&mut self);
}

pub trait DisplayIrq
{
    fn trigger_irq(&mut self, irq_type: Irq);
}

const DISPLAY_WIDTH: usize = 320;
const DISPLAY_HEIGHT: usize = 240;

pub trait GfxCore
{
    fn render_scanline(&mut self);
}

pub mod debug_core;
pub mod hires_core;