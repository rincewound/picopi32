
#[cfg(test)]
mod hires_core_tests
{
    use rstest::*;
    use core1::{Color, DisplayIrq, GfxCore, hires_core::{EMPTY_ATLAS_ID, GfxError, HiResCore, RegisterSet}};
    use std::{cell::RefCell};

     pub struct FakeDisplay
     {

     }

     impl core1::Display for FakeDisplay
     {
        fn push_pixel(&mut self, color: Color) 
        {
            rendered_pixels.with(|pixels|
            {
                pixels.borrow_mut().push(color);
            });
        }

        fn reset_position(&mut self) 
        {
        
        }

        fn show_screen(&mut self) 
        {
        
        }
     }

    struct FakeIrqData
    {
        line_irq: usize,
        vsync_irq: usize   
    }

    impl FakeIrqData
    {
        pub fn new() -> Self
        {
            Self
            {
                line_irq: 0,
                vsync_irq: 0
            }
        }
    }

    thread_local!(static irqData: RefCell<FakeIrqData> = RefCell::new(FakeIrqData::new()));
    thread_local!(static register_memory: RefCell<[u8; 1024 * 32]> = RefCell::new([0; 1024*32]));
    thread_local!(static ram: RefCell<[u8; 1024 * 32]> = RefCell::new([0; 1024 * 32]));

    thread_local!(static rendered_pixels: RefCell<Vec<Color>> = RefCell::new(Vec::new()));

    pub struct FakeIrq
    {
    }

    impl DisplayIrq for FakeIrq
    {
        fn trigger_irq(&mut self, irq_type: core1::Irq) 
        {            
            irqData.with(|data| {
                match irq_type
                { 
                    core1::Irq::Scanline { scanline_index } => {(*data.borrow_mut()).line_irq += 1}
                    core1::Irq::VSync => { (*data.borrow_mut()).vsync_irq += 1}
                    core1::Irq::DmaComplete => {}                    
                }
            });
        }
    }

    pub fn get_reg_ptr() -> *mut RegisterSet
    {
        unsafe 
        {
            let memory_address = register_memory.with(|data|{return data.borrow_mut().as_mut_ptr();});
            return std::mem::transmute::<*mut u8, *mut RegisterSet>(memory_address);
        }
    }

    pub fn get_ram_ptr(offset: usize) -> *const u8
    {
        ram.with(|data|
        {
            let borrow = data.borrow();
            let theref: *const u8 = &borrow[offset] as *const u8 ;
            return theref
        })
    }

    pub fn hirescore () -> HiResCore<FakeIrq, FakeDisplay>
    {
        let memory_address = register_memory.with(|data|{return data.borrow_mut().as_mut_ptr();});
        let irq = FakeIrq {};
        let dsp = FakeDisplay{};
        return HiResCore::<FakeIrq, FakeDisplay>::new(irq, dsp, memory_address);
    }

    pub fn make_pixel_data()
    {
        ram.with(|ram_cell|
        {
            let mut borrow = ram_cell.borrow_mut();
            // we make a 32 x 32 pixel gradient, starting at 0x00
            for i in 0..32
            {
                for i2 in 0..32
                {
                    let offset = i * 32 + i2;
                    borrow[offset] = (1 + i * 2)  as u8;
                }
            }

            // Further, we make a palette at 32x32 + 1
            let palette_start = 32 * 32 + 1;
            for i in 1..64      // don't touch color 0 , this is transparent
            {
                let offset = palette_start + (i * 3);
                borrow[offset + 0] = i as u8;
                borrow[offset + 1] = 2*i as u8;
                borrow[offset + 2] = 3*i as u8;
            }
        });
    }

    pub fn clear_mem()
    {
        ram.with(|ram_cell|
            {
                let mut borrow = ram_cell.borrow_mut();
                // we make a 32 x 32 pixel gradient, starting at 0x00
                for i in 0..32
                {
                    for i2 in 0..32
                    {
                        let offset = i * 32 + i2;
                        borrow[offset] = 0  as u8;
                    }
                }
    
                // Further, we make a palette at 32x32 + 1
                let palette_start = 32 * 32 + 1;
                for i in 1..64      // don't touch color 0 , this is transparent
                {
                    let offset = palette_start + (i * 3);
                    borrow[offset + 0] = 0 as u8;
                    borrow[offset + 1] = 0 as u8;
                    borrow[offset + 2] = 0 as u8;
                }
            });      
    }

    pub fn get_reg_ref() -> &'static mut RegisterSet
    {
        let reg_set = get_reg_ptr();
        unsafe 
        {
            return &mut *reg_set;
        }
    }

    pub fn make_default_data()
    {
        make_pixel_data();
        let regref = get_reg_ref();
        
        regref.layers[0].tilex = 16;
        regref.layers[0].tiley = 16;
        regref.layers[0].tiles[0].atlas_id = 1;
        regref.layers[0].tiles[0].tile_id = 1;
        regref.layers[0].tiles[0].palette_id = 0;
        regref.palettes[0] = get_ram_ptr(32*32 + 1);
        regref.pixel_atlasses[1].data = get_ram_ptr(0);
        regref.pixel_atlasses[1].sizex = 32;
        regref.pixel_atlasses[1].sizey = 32;
        regref.pixel_atlasses[1].storage_mode = core1::hires_core::StorageMode::EightBit;
        regref.lastErr = 0;

    }

    #[rstest]
    pub fn will_trigger_vsync_after_240_lines()
    {
        let mut core = hirescore();

        for _ in 0..239
        {
            core.render_scanline();
            let line_irq = irqData.with(|data|{ return (*data.borrow()).vsync_irq; });
            assert!(line_irq == 0);
        }

        core.render_scanline();
        let line_irq = irqData.with(|data|{ return (*data.borrow()).vsync_irq; });
        assert!(line_irq == 1);
    }

    #[rstest]
    pub fn will_trigger_lineend_irq_if_enabled()
    {
        let mut core = hirescore();
        let mut reg_set = get_reg_ref();
        reg_set.LENDIrqEnable = true;
        reg_set.LYXIrqEnable = true;
        reg_set.LYCCompare = 2;
        core.render_scanline();
        let mut line_irq = irqData.with(|data|{ return (*data.borrow()).line_irq; });
        assert!(line_irq == 0);
        core.render_scanline();
        line_irq = irqData.with(|data|{ return (*data.borrow()).line_irq; });        
        assert!(line_irq == 1);
    }

    #[rstest]
    pub fn will_render_correct_value()
    {
        let mut core = hirescore();
        make_default_data();
        core.render_scanline();
        // At this point we should should find the color 0x01/0x02/0x03 at position 0 in the display:
        let color = rendered_pixels.with(|pxl| pxl.borrow()[0]);
        assert!(color.r == 1 && color.g == 2 && color.b == 3);
    }

    #[rstest]
    pub fn will_render_green_output_if_nothing_was_setup()
    {
        let mut core = hirescore();
        clear_mem();
        core.render_scanline();
        let color = rendered_pixels.with(|pxl| pxl.borrow()[0]);
        assert!(color.r == 0 && color.g == 255 && color.b == 0);  
    }

    #[rstest]
    pub fn will_set_error_if_bad_atlas_id()
    {
        let mut core = hirescore();
        let mut reg_set = get_reg_ref();
        clear_mem();
        make_default_data();
        reg_set.layers[0].tiles[0].atlas_id = 1;
        core.render_scanline();
        assert!(reg_set.lastErr == GfxError::BadAtlasPtr as u8)

    }

    
    #[rstest]
    pub fn will_set_error_if_bad_tile_id()
    {
        let mut core = hirescore();
        let mut reg_set = get_reg_ref();
        clear_mem();
        make_default_data();
        reg_set.layers[0].tiles[0].tile_id = 255;
        core.render_scanline();

        assert!(reg_set.lastErr == GfxError::BadAtlasSize as u8);

        // nothing should be rendered in this case as well:
        let has_rendered = rendered_pixels.with(|pxl| pxl.borrow().len() != 0);
        assert!(has_rendered == false);
    }

    #[rstest]
    pub fn will_render_sprite()
    {
        let mut core = hirescore();
        let mut reg_set = get_reg_ref();
        clear_mem();
        make_default_data();
        reg_set.layers[0].tiles[0].atlas_id = EMPTY_ATLAS_ID;
        reg_set.sprites[0].atlas_id = 1;
        reg_set.sprites[0].atlasx = 4;
        reg_set.sprites[0].atlasx = 4;
        reg_set.sprites[0].w = 8;
        reg_set.sprites[0].h = 8;
        reg_set.sprites[0].posx = 0;
        reg_set.sprites[0].posy = 0;

        core.render_scanline();
        let color = rendered_pixels.with(|pxl| pxl.borrow()[0]);
        assert!(color.r == 1 && color.g == 2 && color.b == 3);  
    }
}