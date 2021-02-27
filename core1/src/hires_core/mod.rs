/*
The hires core is the default gfx core and allows the use of up to 128 sprites of
arbitrary sizes as well as up to 4 layers

Layer 0:
The first layer is the foreground layer. Contents of this layer will be drawn over sprites

Layer 1 - 3:
The background layers

Each layer consists of individual tiles, that are all uniform in size for a given layer. Each
layer can contain up 40 x 30 Tiles

# The Tilemap
A tilemap of a layer stores a 2 byte index into the tileatlas indicating which graphic is displayed
for a given tile. 


# Graphics & Palettes
Pixeldata is stored seperate from palette data to allow the creation of bulk pixeldata. Each graphic
can either use 8 or 4 Bit colours, i.e. either 256 or 8 colors. The storagemode is part of the bulkdata.
In 8 Bit mode, each byte of graphicdata represents a single pixel in the final image. In 4 Bit mode each
byte represents 2 pixels in the final image.

Each pixelvalue indexes into the associated palette, the core will render the color at the palettelocation,
when the pixel is processed.
e.g.
Mode = 4 Bit
Pixels:  1, 1, 3, 2
Palette:
0 = RGBA(0,0,0,0)
1 = RGB(255, 0, 0) // Red
2 = RGB(0, 255, 0) // Green
3 = RGB(0, 0, 255) // Blue
...

Resulting Pixel Values:
Reg, Red, Blue, Green

Note, that for all palettes index 0 denotes the transparent color. Pixels with this color are never rendered
at all.

# Storage of pixeldata
All pixeldata is stored uncompressed either in RAM or ROM. The core must be able to access the data using a
simple address operation. The Layout is as follows:

8 Bit Layout:
Adress  Data
0x0100  0x00 0x01 0x01 0x01 
0x0104  0x00 0x02 0x02 0x02 
0x0108  0x00 0x03 0x03 0x03 

The above data encodes a gradient in 4 x 3 Pixels where the gradient runs from color index 0x01 to index 0x03 with 
a 1 pixel transparent border on the leftside.

The same image as 4 Bit layout:
Adress  Data
0x100   0x01 0x11
0x102   0x02 0x22
0x104   0x03 0x33

# Storage of palettes
Each palette is a continuos range of memory, that is divided into 3 byte chunks, where each chunk is interpreted
as an RGB value. Palettes have no metadata associated to them and are treated as raw memory to allow tricks such
as palette rotation or similar.

To limit the memory use of tiles each tile has a 1 byte palette index, which means, that we can use at most 256
palettes at any given time.

All pixeldata is stored in pixel atlasses. Each atlas contains up to 256 tiles. Note that the core assumes uniform
tilesizes for each atlas, i.e. if the layer is configured to use 20x20 tiles, tile 5 in an atlas will start at pixel
100 of the atlas. An atlas may be of arbitrary size, however it can only ever contain 256 background tiles. Atlasses
are shared with sprites.

All the above leads to the following struct that describes a tile:

struct Tile
{
    atlas_id: u8,
    tile_id: u8
    palette_id: u8,
    RFU: u8         // For alignment/performance purposes, will be used later
}
Size: 4 Byte

and further to the following struct that describes a layer:

struct Layer
{
     tilex: u8,
     tiley: u8,
     scrollx: u16,      // in pixels
     scrolly: u16       // in pixels
     Tiles: [Tile; 40*30]
}

Size: 4808 Byte (incl Padding)

and this is for the atlas:

struct PixelAtlas
{   
    address: ptr
    sizex: usize
    sizey: usize
    storagemode: storagemode    // this can bei either StorageMode::4Bit (=0) oder StorageMode::8Bit(=1)
}
Size: 16 Byte (incl Padding)

Sprites:
A sprite is a hunk of pixeldata that has a size and a location attached. Sprites reference data from a pixelatlas

struct Sprite
{
    x: usize    // screen size!
    y: usize
    w: usize
    h: usize    
    atlasx: usize
    atlasy: usize
    atlasw: usize
    atlash: usize
    atlas_id: u8
    RFU:      u8
    RFU:      u8
    RFU:      u8
}

Size: 64 Byte


-> A difference between w/h and altasw/atlash will cause the core to interpolate. It will always use nearest neighbor interpolation
-> Note that there is no guaranteed order in which sprites are drawn. The core will sort the sprites by y coordinate in order to
   optimize rasterization. The x coordinate is ignored here. When resolving the color for a given pixel the core will use the first
   sprite that occupies the pixel and has a non-transparent color for the location in question.

Implementation Details for the RP2040
- The core will push an event to the IPC FIFO in the following cases:
    * A scanline was finished (EventCode = 1)
    * A frame was finished (EventCode = 2)
    * The scanline set in LYCCompare was finished (EventCode = 3)


ShiftRegisters
--------------
The core exposes two shift registers which are used to shift a pixel in either direction, effectively for allowing special effects, e.g.:
xshift = 1 --> to resolve the color for pixel 10/10, the core will use the coordinate 11/10. 

PixelClock, Timing & Output
---------------------------
The core behaves like a CRT screen in that it renders the image scanline wise and will trigger an interrupt each time a scanline is finished.
Afterwards it will enter a virtual "HBlank" period during which it will not access any registers. After each frame it will enter a virtual "VBlank",
during it will not access registers (except the DMA register) and process any DMA requests.

The goal is to output 25 - 30 fps @ 320x240 pixels. This leaves at 40 - 33ms per frame, however we'll want to add points during which the other core
can interact with the gfx unit.
* After each scanline an interrupt is generated and the core simulates a 10 ns HBlank (i.e. 1330 CPU cylces on the RP2040) after which the core will continue onto the next scanline. The RP2040 Cortex M0 cores have
an IRQ latency of ~ 16 cylces, coupled with context switch cost the other core can probably execute 1000 cycles worth of instructions during HBlank.
This leaves us with 0.147 ms/line * 240 lines = 35.28 ms/frame.
* After each frame a 5 ms VBlank is simulated. This equals 665k cylcles on the RP2040

These pauses add up to 5 ms VBlank + 10ns * 240 lines = 7.4 ms, leaving us with 32.6 - 25.6 ms per frame for outputting pixels in order to reach the 25-30
fps goal. The pixelclock calculates as:
(320 * 240) / 25.6 = 3000 pixel/ms, with each pixel = 2 byte -> 6000 byte/ms -> 6.000.000 byte/s ~ 48   MHz SPI clock
(320 * 240) / 32.6 = 2355 pixel/ms, with each pixel = 2 byte -> 4710 byte/ms -> 4.710.000 byte/s ~ 37.7 MHz SPI clock

CPU Clocks per Pixel then equals:
133 000 cycles/ms / 3000 pixl/ms  = ~28 cycles per Pixel
-> We need about 2 cycles to load a halfword
-> Simple arithmetic ops (ie. add/sub) take 1 cylce
-> We sadly cannot do blits due to:
    * Palettes needing an inderection to resolve color
    * Transparency requiring us to look at each pixel

* The targetdisplay uses 16 bit color!

* We assume this works, otherwise we will have to fallback to monochrome, i.e. 1 Bit/pixel.

Virtual Registers & DMA
-----------------------
The core's functionality is exposed by means of virtual registers that are mapped to a wellknown memory address. The consuming core / app can read and
write to the registers. The core does not expose direct access to layerdata. Instead a pseudo DMA mechanism is used, where the consuming core sets up
a transfer and a target and triggers the DMA by means of sending a DMA Transfer Request to the core.

Complete Registerset
--------------------

GfxControlRegister (RP2040 0x20000 TBD!)
{
    OutputEnable: boolean       // Toggles on screen output
    LENDIrqEnable: boolean      // Toggles line end interrupt
    LYXIrqEnable: boolean       // Toggles the Line Comparator Interrupt
    RFU:          u8
    LYCCompare:   u16           // Line to trigger the Line Comparator at
    xshift:       u16
    yshift:       u16
    RFU:          u8
    RFU:          u8
    pixel_atlasses: [PixelAtlas; 16]       // stores memory address for each all atlases
}

Size: 268 Byte

DmaControlRegister (RP2040 0x20020 TBD!)
{
    source_address: usize
    target_address: usize
    size:           usize
}

Size: 6 Byte

SpriteControlReg (RP2040 0x20020 TBD!)
{
    sprites:   [Sprite; 128]
}
Size 8192 Byte

LayerControlRegister
{
    layers: [Layer; 4]
}
19200 Byte


Total:
The core needs about 27 KiB of RAM for resources, this excludes 
any workram and actual graphics.

Using the core
* During normal operation the core will emit a VSync IRQ after each rendered frame,
  use the ISR to setup DMA transfers using the DMAControlRegister.
* Use DMA Transfers to setup backgroundlayers and sprite properties en bulk.
* Use single writes to change few thing
* Don't violate the 5 ms VBlank, as it will result in undefined behavior
* Don't write to memoryareas that are affected by a pending DMA Transfer
* Load all graphics to RAM before use, QSPI is usable with it's own address region but
  it will probably introduce performance problems

*/

use crate::Display;
use crate::DisplayIrq;
use crate::GfxCore;

pub struct RegisterSet
{
    
    pub OutputEnable: bool,      // Toggles on screen output
    pub LENDIrqEnable: bool,     // Toggles line end interrupt
    pub LYXIrqEnable: bool,     // Toggles the Line Comparator Interrupt
    pub RFU0:          u8,
    pub LYCCompare:   u16,         // Line to trigger the Line Comparator at
    pub xshift:       u16,
    pub yshift:       u16,
    pub RFU1:          u8,
    pub RFU2:          u8,
}

pub struct HiResCore<I: DisplayIrq>
{
    display_irq: I,
    scanline: u16,
    registers: *mut RegisterSet
}

impl<I: DisplayIrq> HiResCore<I>
{
    pub fn new(display_irq: I, register_adr: *mut u8) -> Self
    {
        unsafe
        {
            let adr = core::mem::transmute::<*mut u8, *mut RegisterSet>(register_adr);
        
            return Self{
                display_irq,
                scanline: 0,
                registers: adr
            };
        }
    }
}

impl <I: DisplayIrq> GfxCore for HiResCore<I>
{
    fn render_scanline(&mut self) {
        self.scanline += 1;

        unsafe 
        {
            if (*self.registers).LYXIrqEnable && (*self.registers).LYCCompare == self.scanline
            {
                self.display_irq.trigger_irq(super::Irq::Scanline{scanline_index: self.scanline as usize});
            }
        }

        if self.scanline > 239
        {
            self.display_irq.trigger_irq(super::Irq::VSync);
            self.scanline = 0;
        }
    }
}