use core1::GfxCore;
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
    unsafe 
    {
        return register_memory.with(|data|{return data.borrow_mut().as_mut_ptr();});
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