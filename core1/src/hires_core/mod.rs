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
     tilex: u8,         // TBD: This should be a power of 2
     tiley: u8,         // TBD: This should be a power of 2
     scrollx: u16,      // in pixels
     scrolly: u16       // in pixels
     Tiles: [Tile; 40*30]
}

Size: 4808 Byte (incl Padding)

and this is for the atlas:

struct PixelAtlas
{   
    address: ptr
    sizex: usize                // TBD: This should be a power of 2
    sizey: usize                // TBD: This should be a power of 2
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

Graphics Modes
--------------
The core supports 2 Modes:
- Mode 0: All layers are available "normally"
- Mode 1: A pseudo textmode, in which Layer 0 is setup as textlayer and uses the internal character rom for display.

In Mode 1, pixel_atlas 0 will be setup to point to the character rom, 
further, Palette 0  will be the pixel atlasses palette.


Complete Registerset
--------------------

GfxControlRegister (RP2040 0x20000 TBD!)
{
    OutputEnable: boolean       // Toggles on screen output
    LENDIrqEnable: boolean      // Toggles line end interrupt
    LYXIrqEnable: boolean       // Toggles the Line Comparator Interrupt
    Mode:          u8           // Mode the core is in
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
* Use the outputenable register to disable the core if you need an elaborate setup
  routine.


Using the core for the simulation target:
* The core takes a raw pointer to a preallocated bit of memory. It will store its
virtual registers there. To use the core in the simulation use a preallocated bit of
memory, convert it into a ptr and pass it to the core.

!! Attention !! This will obivously involve unsafe code, be extra aware when setting up the simulation !!

Error handling:
The core exposes an error register. It will store all errors in that register. Further, it
will stop rendering as long as a value != 0 is in the register.

# Effects
With the current design a couple of effects can be archieved easily, 

## Palette Rotation
By shifting the startpointer of a palette along its memory a palette rotation effect
can be created. If you attempt to do this make sure, that the graphics the rotation is
applied to don't use the full 256 entries of the palette.

## Screenwobble
Use the Scanline Interrupt to set the X scrolloffset

## Parallax Scrolling
...is easily archieved by using layers with different scroll offsets.

*/

use core::u16;

use crate::{Color, Display};
use crate::DisplayIrq;
use crate::GfxCore;

use self::character_rom::{character_rom_data, character_rom_pal};

mod character_rom;

const MAX_PALETTES: usize = 32;
const NUM_LAYERS: usize = 4;
const MAX_ATLASSES: usize = 16;
const MAX_SPRITES: usize = 64;

const SPRITE_LAYER: usize = 1;

pub const EMPTY_ATLAS_ID: u8 = 255;

/*
Layers:
0 -- Foreground
1 -- Sprites
2 -- BG0
3 -- BG1
*/

#[repr(u8)]
pub enum GfxError
{
    Ok,
    BadAtlasId,     // Tried to access an atlas with an invalid id
    BadAtlasPtr,    // Atlas has no mem assigned
    BadAtlasSize,   // Tried to retrieve an entry that was outside of atlas bounds
    BadStorageMode, // Encountered atlass with invalid storage mode
    BadPaletteId,   // Tried to use a palett with an invalid id
    BadPalettePtr,  // Palette has no mem assigned    
}

#[derive(PartialEq)]
pub enum StorageMode
{
    FourBit,
    EightBit
}

pub struct PixelAtlas
{
    pub data: *const u8,
    pub sizex: u16,
    pub sizey: u16,
    pub storage_mode: StorageMode
}

#[derive(Copy, Clone)]
pub struct Tile
{
    pub atlas_id: u8,
    pub tile_id: u8,
    pub palette_id: u8,
    _rfu: u8
}

pub struct Layer
{
    pub tilex: u8,
    pub tiley: u8,
    pub scrollx: u16,
    pub scrolly: u16,
    pub tiles: [Tile; 40*30]
}

pub struct Sprite
{
    pub posx: i16,
    pub posy: i16,
    pub w: i16,
    pub h: i16,
    pub palette_id: u8,
    pub atlas_id: u8,
    pub atlasx: u16,
    pub atlasy: u16
}

#[repr(C)]
pub struct RegisterSet
{  
    pub output_enable: bool,      // Toggles on screen output
    pub LENDIrqEnable: bool,     // Toggles line end interrupt
    pub LYXIrqEnable: bool,     // Toggles the Line Comparator Interrupt
    pub Mode:          u8,
    pub LYCCompare:   u16,         // Line to trigger the Line Comparator at
    pub xshift:       u16,
    pub yshift:       u16,
    pub RFU1:          u8,
    pub RFU2:          u8,
    pub lastErr:       u8,      // Contains the last encountered error
    pub pixel_atlasses: [PixelAtlas; MAX_ATLASSES],
    pub layers: [Layer; NUM_LAYERS],
    pub palettes: [*const u8; MAX_PALETTES],
    pub sprites: [Sprite; MAX_SPRITES]
}

pub struct HiResCore<I: DisplayIrq, D: crate::Display>
{
    display_irq: I,
    display: D,
    scanline: u16,
    registers: *mut RegisterSet,
    active_mode: u8
}

impl<I: DisplayIrq, D: crate::Display> HiResCore<I, D>
{
    pub fn enable_text_mode(&mut self)
    {
        // The active_mode fiels is used to make sure we only enable
        // the char_rom once and not every cycle, when we see the Mode register
        // being one. This saves some processing time and also allows us to -e.g.-
        // change the palette for the characters, if we desire to do so.
        if self.active_mode != 1
        {
            let regs = self.get_registers_mut();
            regs.pixel_atlasses[0].data = &character_rom_data as *const u8;
            regs.pixel_atlasses[0].sizex = 128;
            regs.pixel_atlasses[0].sizey = 192;

            unsafe
            {
                regs.palettes[0] = core::mem::transmute::<*const u32, *const u8>(&character_rom_pal as *const u32);
            }

            regs.layers[0].tilex = 8;
            regs.layers[0].tiley = 8;

            for i in 0..(30*40)
            {
                regs.layers[0].tiles[i].palette_id = 0;
                regs.layers[0].tiles[i].atlas_id = 0;
            }
            regs.Mode = 1;
            self.active_mode = 1;
        }

    }
}

impl<I: DisplayIrq, D: crate::Display> HiResCore<I, D>
{
    pub fn new(display_irq: I, display: D, register_adr: *mut u8) -> Self
    {
        unsafe
        {
            let adr = core::mem::transmute::<*mut u8, *mut RegisterSet>(register_adr);
        
            let result = Self{
                display_irq,
                display,
                scanline: 0,
                registers: adr,
                active_mode: 0
            };
            result
        }
    }

    pub fn get_registers_mut(&mut self) -> &mut RegisterSet
    {
        unsafe 
        {
            return &mut *self.registers;
        }
    }

    pub fn get_registers(&self) -> &RegisterSet
    {
        unsafe 
        {
            return &*self.registers;
        }
    }

    fn get_atlas_data(&self, atlas_id: u8, offset: usize) -> Result<u8, GfxError>
    {
        if atlas_id > MAX_ATLASSES as u8
        {
            return Err(GfxError::BadAtlasId);
        }

        let atlas = &self.get_registers().pixel_atlasses[atlas_id as usize];
        let data_ptr = atlas.data;
        if data_ptr.is_null()
        {
            return Err(GfxError::BadAtlasPtr);
        }
        
        // Calculate the  byteoffset into the atlas. This is dependent
        // on the atlas storage type, as for 4 bit we have 2 pixels per
        // byte
        let byte_offset: usize;
        let pixels_in_atlas: usize;
        if atlas.storage_mode == StorageMode::EightBit
        {
            byte_offset = offset;
            pixels_in_atlas = atlas.sizex as usize * atlas.sizey as usize;
        }
        else if atlas.storage_mode == StorageMode::FourBit
        {            
            byte_offset = (offset & 0xFFFFFFFE) / 2;
            pixels_in_atlas = atlas.sizex as usize * atlas.sizey as usize * 2;
        } 
        else
        {
            // errorcase, since we're potentially dealing
            // with raw memory, we can't use a match, as we
            // possible find values that don't exist in the
            // enum. In this case we err out.
            return Err(GfxError::BadStorageMode);
        }

        if byte_offset > pixels_in_atlas
        {
            return Err(GfxError::BadAtlasSize);
        }

        unsafe 
        {
            let atlas_byte = *data_ptr.offset(byte_offset as isize);

            if atlas.storage_mode == StorageMode::EightBit
            {
                return Ok(atlas_byte);
            }

            let the_byte: u8;
            if offset & 0x01 != 0x00
            {
                the_byte = atlas_byte &0x0F;
            }
            else
            {
                the_byte = atlas_byte >> 4
            }
            return Ok(the_byte);

        }
    }

    fn get_colorindex_from_atlas(&self, atlas_id: u8, tile_id: u16, tilew: u8, tileh: u8, pixelx: u16, pixely: u16) -> Result<u8, GfxError>
    {
        /*
            How this works:
            Each atlas, when used as a tilesource, is divided up into evenly sized chunks (e.g. 8x8, 10x4...),
            i.e. a 50x50 atlas can be chunked into 10x10 tiles, where each tile is 5x5 pixels.
            To obtain the actual color for a pixel within a tile we use the tile id and the tilesizes as well as
            the offset into the tile to...

        */
        let atlas = &self.get_registers().pixel_atlasses[atlas_id as usize];

        if atlas.sizex == 0 || atlas.sizey == 0
        {
            return Err(GfxError::BadAtlasId);
        }

        let atlas_width = atlas.sizex / (tilew as u16);

        // ... calculate which column from the tileid
        let tile_col = tile_id % atlas_width;
        
        // ... calculate at which (pixel!) offset into the atlas the row of our tile starts
        let tile_row_start = (tile_id - tile_col) * (tilew as u16 * tileh as u16) as u16;

        // ... calculate the actual start of the tiledata. Note that tiledata is not arranged sequentially, but
        //     as a matrix (i.e. it is a subset of an image!). We also incorporate the y position of the pixel
        //     we're looking for. Since our tile is a subset of an image we have to add y * sizex of the atlas
        //     to the start of the tiledata.
        let intra_tile_start = tile_row_start + (tile_col * tilew as u16) + pixely * atlas.sizex as u16;
        let offset = (intra_tile_start + pixelx) as usize;

        // note that the offset is a pixeloffset here. get_atlas_data will take the actual storage mode
        // into account.
        let atlas_data = self.get_atlas_data(atlas_id, offset)?;

        return Ok(atlas_data);        
    }

    fn retrieve_palette_entry(&self, palette_id: u8, entry_id: u8) -> Result<Color, GfxError>
    {
        if entry_id as usize > MAX_PALETTES
        {
            return Err(GfxError::BadPaletteId);
        }
        let palette = &self.get_registers().palettes[palette_id as usize];
        if palette.is_null()
        {
            return Err(GfxError::BadAtlasPtr);
        }

        unsafe 
        {
            let start_ptr = palette.offset(((4 * entry_id)) as isize);
            let v0 = *start_ptr;
            let v1 = *(start_ptr.offset(1));
            let v2 = *(start_ptr.offset(2));
            let v3 = *(start_ptr.offset(3));

            // Little Endian storage by palette tool --> stoopid!
            // let b = palette.offset(((4 * entry_id)) as isize); 
            // let g = b.offset(1);
            // let r = b.offset(2);
            // let r = palette.offset(((4 * entry_id) +1) as isize);      //An entry from sprtool is 4 bytes wide!
            // let g = r.offset(1);
            // let b = r.offset(2);
            return Ok(Color{r: v2, g: v1, b: v0 + 0*v3});
        }        
    }

    fn get_layer_pixel(&mut self, pixel_index: u16, layer_index: usize) -> Result<Option<Color>, GfxError>
    {
        let current_scanline = self.scanline;
        let atlas_id;
        let tile_id;
        let tile_pixel_x;
        let tile_pixel_y ;
        let palette_id;
        let tile_src_w ;
        let tile_src_h;
        let the_tile;
        {
            let regs = self.get_registers_mut();

            if regs.layers[layer_index].tilex == 0 || regs.layers[layer_index].tiley == 0
            {
                return Ok(None);
            }

            /*
                Note: This innerloop is quite expensive as it requires
                4 divisions to perform the lookup of the tile. We could
                replace them with bitshifts and bitwise logic, if we
                opt to only ever use  power of 2 graphic sizes for
                pixel atlasses and tiles!
            */

            // calculate correct tile in this layer
            let row = current_scanline / regs.layers[layer_index].tiley as u16;
            let col = pixel_index as u16 / regs.layers[layer_index].tilex as u16;
            tile_id = row * 40 + col as u16;
            tile_pixel_x = pixel_index  as u16 % regs.layers[layer_index].tilex as u16;
            tile_pixel_y = current_scanline % regs.layers[layer_index].tiley  as u16;
            the_tile = regs.layers[layer_index].tiles[tile_id as usize];
            // we now have the palette entry for the pixel i tile_pixel, 
            // retrieve the actual color:
            palette_id = the_tile.palette_id;
            atlas_id = the_tile.atlas_id;
            tile_src_w = regs.layers[layer_index].tilex;
            tile_src_h = regs.layers[layer_index].tiley;

            if atlas_id == EMPTY_ATLAS_ID
            {
                return Ok(None);
            }

        }

        let color_index = self.get_colorindex_from_atlas(atlas_id, the_tile.tile_id as u16, tile_src_w, tile_src_h, tile_pixel_x, tile_pixel_y);
        if let Ok(col) = color_index
        {
            if col == 0     // 0 is the transparent color index and is never rendered!     
            {
                return Ok(None);
            }                              

            let the_color = self.retrieve_palette_entry(palette_id, col);
            if let Ok(res_color) = the_color
            {
                if res_color.r == 255 && res_color.g == 0 && res_color.b == 255 
                {
                    return Ok(None);
                }
                return Ok(Some(res_color))
            }
            else
            {
                return Err(the_color.unwrap_err());
            }
        }
        else
        {
            return Err(color_index.unwrap_err());
        }
        
    }

    
    fn get_sprite_pixel(&self, pixel_index: u16) -> Result<Option<Color>, GfxError>
    {
        let regs = self.get_registers();
        let scanline = self.scanline;
        for sprite_id in 0..MAX_SPRITES
        {
            // check if sprite is used:
            let sprite = &regs.sprites[sprite_id];

            // effectively: 0xFFFF in both pos registers mark sprite as unused!
            if sprite.posx == -1i16 && sprite.posy == -1i16     
            {
                continue;
            }
            
            //hitbox test for sprite:
            if !((sprite.posx <= pixel_index  as i16) && (pixel_index as i16) < (sprite.posx + sprite.w))
            {
                continue;
            }
            if !((sprite.posy <= scanline as i16) && (scanline as i16) < (sprite.posy + sprite.h))
            {
                continue;
            }

            if sprite.atlas_id > MAX_ATLASSES as u8
            {
                return Err(GfxError::BadAtlasId);
            }

            

            // calculate position within the sprite's pixelatlas:
            let atlas = &regs.pixel_atlasses[sprite.atlas_id as usize];    

            let sprite_pixel_x = pixel_index as i16 - sprite.posx;        
            let sprite_pixel_y = scanline as i16 - sprite.posy;            
            
            let atlas_x = sprite.atlasx + sprite_pixel_x as u16;
            let atlas_y = sprite.atlasy + sprite_pixel_y as u16;
            // calculate offset into atlas:
            let atlas_offset = atlas_y * atlas.sizex + atlas_x;
            let color_index = self.get_atlas_data(sprite.atlas_id, atlas_offset as usize);
            
            if let Ok(color) = color_index
            {
                // Fix this: Sprite tool generates this wrong
                // if color == 0       // Transparent
                // {
                //     // If we're transparent, we check other sprites
                //     // that might occupy this pixel.
                //     continue;
                // }
                let actual_color = color;

                let color_entry = self.retrieve_palette_entry(sprite.palette_id, actual_color);
                if let Ok(color) = color_entry
                {         
                    if color.r == 255 && color.g == 0 && color.b == 255 
                    {
                        return Ok(None);
                    }
                    return Ok(Some(color));
                }
                else
                {
                    return Err(color_entry.unwrap_err());
                }

            }
            else
            {
                return Err(color_index.unwrap_err());
            }
        }
        
        return Ok(None);
    }
    
    fn render_scanline_pixels(&mut self)
    {

        for pixel in 0..crate::DISPLAY_WIDTH
        {
            /* Resolve pixelcolor, check all layers: */
            let mut pixel_rendered = false;
            for i in 0..NUM_LAYERS
            {
                let mut color;
                if i == SPRITE_LAYER
                {
                    // For the sprite layer we first check, if a sprite generates
                    // a pixel, afterwards we check the layercontent
                    color = self.get_sprite_pixel(pixel as u16);
                    if let Ok(col) = color
                    {
                        if col.is_none()
                        {
                            color = self.get_layer_pixel(pixel as u16, i);
                        }
                    }
                }
                else
                {
                    color = self.get_layer_pixel(pixel as u16, i);
                }

                if let Ok(output_pixel) = color
                {
                    if let Some(pixel_color) = output_pixel
                    {
                        self.display.push_pixel(pixel_color);
                        pixel_rendered = true;
                        break;
                    }
                }
                else
                {
                    self.get_registers_mut().lastErr = color.unwrap_err() as u8;
                    // We abort this scanline immediately, thisway the "user" should
                    // be able to spot the pixel where things went wrong
                    return;
                }                
            }

            /* 
                This pixel could not be assigned a color. This can have a variety of reasons,
                e.g. no layer returned a color value (all returned  index 0) or no layer was
                setup in the first place.
                Since we have to output something, we use a signal value (bright green!)
             */
             if !pixel_rendered
             {

                self.display.push_pixel(Color{r:0, g: 255, b:0});
             }
        }

    }
}

impl <I: DisplayIrq, D: Display> GfxCore for HiResCore<I,D>
{
    fn render_scanline(&mut self) {

        let regs = self.get_registers();
        if regs.lastErr != 0
        {
            /* 
                If we encountered an error in a previous frame
                we don't generate output at all. The other core
                is responsible for setting us up correctly.
            */
            return;
        } 

        // if regs.output_enable == true
        // {
        //     /* only if the other core enabled the output, we render
        //        anything.
        //     */
        //     return;
        // }

        
        self.render_scanline_pixels();

        let regs = self.get_registers_mut();

        let line_end_enable = regs.LENDIrqEnable;

        if regs.LYXIrqEnable && regs.LYCCompare == self.scanline
        {
            self.display_irq.trigger_irq(super::Irq::Scanline{scanline_index: self.scanline as usize});
        }

        if line_end_enable
        {
            self.display_irq.trigger_irq(super::Irq::Scanline{scanline_index: self.scanline as usize});
        }

        self.scanline += 1;

        if self.scanline > (crate::DISPLAY_HEIGHT - 1) as u16
        {
            self.display_irq.trigger_irq(super::Irq::VSync);
            self.scanline = 0;
        }
    }



    fn render_frame(&mut self) {

        // Check if we have to enable the character rom:
        let regs = self.get_registers_mut();
        if regs.Mode == 1
        {
            self.enable_text_mode();
        }

        for _ in 0..crate::DISPLAY_HEIGHT
        {
            self.render_scanline();
            // ToDo: Sleep for 1300 ops
        }
        self.display.show_screen();
        // ToDo: Sleep for 660kops
    }
}