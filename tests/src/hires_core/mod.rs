
#[cfg(test)]
mod hires_core_tests
{
    use rstest::*;
    use core1::{DisplayIrq, GfxCore, hires_core::HiResCore};
    use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

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

    pub fn hirescore () -> HiResCore<FakeIrq>
    {
        let irq = FakeIrq {};
        return HiResCore::<FakeIrq>::new(irq);
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
}