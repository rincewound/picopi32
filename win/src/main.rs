use core1::{GfxCore, hires_core::RegisterSet};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::{cell::RefCell, time::Duration};

mod sdl_display;

struct dummy_irq{}

impl core1::DisplayIrq for dummy_irq{
    fn trigger_irq(&mut self, irq_type: core1::Irq) {

    }
}

thread_local!(static register_memory: RefCell<[u8; 1024 * 32]> = RefCell::new([0; 1024*32]));
    
pub fn get_reg_ptr() -> *mut u8
{

    return register_memory.with(|data|{return data.borrow_mut().as_mut_ptr();});
}

pub fn get_reg_set() -> &'static mut RegisterSet
{
    unsafe 
    {
        let memory_address = register_memory.with(|data|{return data.borrow_mut().as_mut_ptr();});
        let mut ptr = std::mem::transmute::<*mut u8, *mut RegisterSet>(memory_address);
        return &mut *ptr;
    }
}

fn write_text(regset: &mut RegisterSet, text: String, row: u16, col: u16)
{
    let mut tilepos = row * 40 + col;
    for char in text.as_bytes().into_iter()
    {                
        regset.layers[0].tiles[tilepos as usize].tile_id = *char;
        tilepos +=1;
        if tilepos > 30*40
        {
            return;
        }
    }
}

pub fn main() {
    
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("picopi-win", 320, 240)
        .position_centered()
        .build()
        .unwrap();
 
    let canvas = window.into_canvas().build().unwrap();
    let dsp = sdl_display::SdlDisplay::new(canvas);
    let irq = dummy_irq{};    
    let mut gfxCore = core1::hires_core::HiResCore::new(irq, dsp, get_reg_ptr());

    let mut regmem = get_reg_set();
    regmem.Mode = 1;
    regmem.sprites[0].atlas_id = 0;
    regmem.sprites[0].atlasx = 10;
    regmem.sprites[0].atlasy = 133;
    regmem.sprites[0].h = 32;
    regmem.sprites[0].w = 110;
    regmem.sprites[0].palette_id = 0;
    regmem.sprites[0].posx = 0;
    regmem.sprites[0].posy = 0;

    /* Show some text */
    write_text(regmem, "PicoPi32 Textmode".to_string(), 10, 5);
    

    regmem.output_enable = true;

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        gfxCore.render_frame();
        regmem.sprites[0].posx += 1;
        regmem.sprites[0].posy += 1;

        if regmem.sprites[0].posx > 200
        {
            regmem.sprites[0].posx = 0
        }

        if regmem.sprites[0].posy > 200
        {
            regmem.sprites[0].posy = 0
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}