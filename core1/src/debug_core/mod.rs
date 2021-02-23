use crate::{Color, Display, DisplayIrq, GfxCore, Irq};


pub struct DebugGfxCore<D, I>
where D: Display, I: DisplayIrq
{
display: D,
irq: I
}


/* This is the most basic possible graphics core - 
it will just produce a magenta image
*/

impl<D: Display, I: DisplayIrq> DebugGfxCore<D, I>
{
    pub fn new(display: D, irq: I) -> Self
    {
        return Self {display, irq};
    }
}

impl <D: Display, I: DisplayIrq> GfxCore for DebugGfxCore<D, I>
{
    fn tick(&mut self)
    {
        for line_index in 0..crate::DISPLAY_HEIGHT
        {
            for _ in 0..crate::DISPLAY_WIDTH
            {
                self.display.push_pixel(Color { r: 255, g: 0, b: 255});
            }
            self.irq.trigger_irq(Irq::Scanline{scanline_index: line_index});
        }
        self.irq.trigger_irq(Irq::VSync);

        self.display.show_screen();
    }
}