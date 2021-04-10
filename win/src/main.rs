use MWalk0::{MWalk0_data, MWalk0_pal};
use core1::{GfxCore, hires_core::RegisterSet};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::{cell::RefCell, time::Duration};

mod sdl_display;
mod MWalk0;

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

pub fn get_sprite_ptr() -> *const u8
{
    let adr = &MWalk0::testpat_data as *const u8;
    return adr;
}

pub fn get_pal_ptr() -> *const u8
{
    unsafe 
    {
        let adr = &MWalk0::testpat_pal as *const u32;
        return std::mem::transmute::<*const u32, *const u8>(adr);
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
    regmem.pixel_atlasses[0].data = get_sprite_ptr();
    regmem.pixel_atlasses[0].sizex = 10;
    regmem.pixel_atlasses[0].sizey = 10;
    regmem.pixel_atlasses[0].storage_mode = core1::hires_core::StorageMode::FourBit;
    regmem.palettes[0] = get_pal_ptr();
    regmem.sprites[0].atlas_id = 0;
    regmem.sprites[0].atlasx = 0;
    regmem.sprites[0].atlasy = 0;
    regmem.sprites[0].h = 10;
    regmem.sprites[0].w = 10;
    regmem.sprites[0].palette_id = 0;
    regmem.sprites[0].posx = 0;
    regmem.sprites[0].posy = 0;

    regmem.OutputEnable = true;

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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}