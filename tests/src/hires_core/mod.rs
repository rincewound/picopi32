
#[cfg(test)]
mod hires_core_tests
{
    use rstest::*;
    use core1::{DisplayIrq, GfxCore, hires_core::{HiResCore, RegisterSet}};
    use std::{cell::RefCell};

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
    thread_local!(static memory: RefCell<[u8; 1024 * 32]> = RefCell::new([0; 1024*32]));

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
            let memory_address = memory.with(|data|{return data.borrow_mut().as_mut_ptr();});
            return std::mem::transmute::<*mut u8, *mut RegisterSet>(memory_address);
        }
    }

    pub fn hirescore () -> HiResCore<FakeIrq>
    {
        let memory_address = memory.with(|data|{return data.borrow_mut().as_mut_ptr();});
        let irq = FakeIrq {};
        return HiResCore::<FakeIrq>::new(irq, memory_address);
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
        let mut reg_set = get_reg_ptr();
        unsafe
        {
            (*reg_set).LENDIrqEnable = true;
            (*reg_set).LYXIrqEnable = true;
            (*reg_set).LYCCompare = 2;
        }
        core.render_scanline();
        let mut line_irq = irqData.with(|data|{ return (*data.borrow()).line_irq; });
        assert!(line_irq == 0);
        core.render_scanline();
        line_irq = irqData.with(|data|{ return (*data.borrow()).line_irq; });        
        assert!(line_irq == 1);
    }
}