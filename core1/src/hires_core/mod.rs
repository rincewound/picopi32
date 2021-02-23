/*
The hires core is the default gfx core and allows the use of up to 128 sprites of
arbitrary sizes as well as up to 6 layers

Layer 0:
The first layer is the foreground layer. Contents of this layer will be drawn over sprites

Layer 1 - 5:
The background layers

Each layer consists of individual tiles, that are all uniform in size for a given layer. Each
layer can contain up 80 x 60 Tiles, allowing to store 2x2 full screens worth of tiles if 8x8 px
tiles are used and a lot more if larger tiles are used.

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

The above data encodes a gradient in 4 x 3 Pixels where the gradien runs from color index 0x01 to index 0x03 with 
a 1 pixel transparent border on the leftside.

The same image as 4 Bit layout:
0x100 0x01 0x11
0x102 0x02 0x22
0x104 0x03 0x33

*/