# Using the HiresCore

## Concepts

### Atlas
An atlas is a a collection of graphics data that consists solely of references into a palette. An atlas can be encoded with either 4 or 8 bit palette size (i.e. 16 or 256 colors).

### Palette
A palette is a collection of colors in RGBA format. A palette per se has no fixed length. A palette entry consists of 4 bytes data, which contain the color information. Colors are sorted back-to-front, i.e. given a pointer to a palette entry, the first byte is the blue channel, then green, then red, the alpha. Note that, while an alpha channel is present, the hirescore ignores this data.

### Layer
The graphics of the hirescore are organized with two core ideas, one being layers. The core uses 4 layers to display static data (e.g. backgrounds!). Each layer is divided up into 40x30 uniform tiles. 

### Tile
A tile is the smallest part of a layer. It basically consists of a reference to an atlas, an offset into the atlas and a reference to a palette that is to be used when displaying the atlas data.

### Sprite

## How the rasterizer works

## Getting stuff onto the screen
To get started simple, we want to display an arbitrary image. To do this we'll need to first prepare the image by extracting the palette and converting it into a pixel atlas. This can be done by means of the sprite tool.

The spritetool generates a rustmodule for an image containing the atlasdata and seperate palette data. Note that the tool will automatically try to find a suitable palette, if the image comes with too many colors.

Assuming we already have created an atlas and a palette for the graphic we want to display.Let's say they are in the module "sprite.rs", the data was encoded with the default StorageMode and is 128x128 pixels in size.

The first step is to setup the core to use our pixeldata. Do this by setting up one of the pixel atlasses in the Registerset to point to our data:
```rust

use self::sprite::{sprite_data, sprite_pal};

fn setup_sprite()
{
    let regs = get_reg_set_mut();      // We assume this function gives uns a mutable reference to the registerset of the core.

    
    // You can use any of the available pixelatlasses here (0..15)
    // We use 1 here, because slot 0 will be used by the character rom
    // if you want to use the textmode in parallel
    regs.pixel_atlasses[1].data = &sprite_data as *const u8;
    regs.pixel_atlasses[1].sizex = 128;
    regs.pixel_atlasses[1].sizey = 128;
}
```

After executing the above code the core is ready to use the atlas. We're still missing the
palette however. This involves some unsafe code, because - at this point, the spritetool emits
u32 values for palettes, while the core expects a pointer to an u8:

```rust
    unsafe
    {
        regs.palettes[1] = core::mem::transmute::<*const u32, *const u8>(&sprite_pal as *const u32);
    }
```

At this point we have all the data required to display our sprite. We just have to use one of the available sprite slots to do so:

```rust
fn setup_sprite()
{
    let regs = get_reg_set_mut();
    regs.sprites[0].posx = 0;       // On-Screen Position X
    regs.sprites[0].posy = 0;       // On-Screen Position Y
    regs.sprites[0].w = 128;        // Width
    regs.sprites[0].h = 128;        // Height
    regs.sprites[0].palette_id = 1; // Index of the palette to use, we setup palette 1, so that's what we want here.
    regs.sprites[0].atlas_id = 1;   // Index of the atlas to use, we setup atlas 1, so that's what we want here.
    regs.sprites[0].atlasx = 0;     // Position of the sprite in the atlas. Since we want to display the whole 128x128 image, this 0/0
    regs.sprites[0].atlasy = 0;
}
```
The spritedata is retained by the core until it is actively changed. The core allows up to 64 sprites to be used at any given time. 


At this point, the core will display the sprite at position 0/0 as soon as it renders a frame. Note that this will only happen, if you set the register "output_enable" to true, otherwise the core will not do anything. This feature is intended to allow for uninterrupted setup of graphics without graphical artifaces.


## Testing in the windows simulation



