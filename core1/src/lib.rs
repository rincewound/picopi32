pub struct Color
{
    pub R: u8,
    pub G: u8,
    pub B: u8
}

pub enum Irq
{
    Scanline{scanline_index: usize},        // The Scanline IRQ is triggered, when the GfxCore has finished drawing a scanline
    VSync                                   // The VSync IRQ is triggered, when the GfxCore has finished drawing a complete image
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

pub struct DebugGfxCore<D, I>
    where D: Display, I: DisplayIrq
{
    display: D,
    irq: I
}

const Display_Width: usize = 320;
const Display_Height: usize = 240;


/* This is the most basic possible graphics core - 
   it will just produce a magenta image
*/
impl<D: Display, I: DisplayIrq> DebugGfxCore<D, I>
{
    pub fn new(display: D, irq: I) -> Self
    {
        return Self {display, irq};
    }

    pub fn tick(&mut self)
    {
        for line_index in 0.. Display_Height
        {
            for _ in 0..Display_Width
            {
                self.display.push_pixel(Color { R: 255, G: 0, B: 255});
            }
            self.irq.trigger_irq(Irq::Scanline{scanline_index: line_index});
        }
        self.irq.trigger_irq(Irq::VSync);

        self.display.show_screen();
    }
}