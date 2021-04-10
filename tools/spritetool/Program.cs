using System;
using System.Collections;
using System.Collections.Generic;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace spritebuilder
{
    enum ColorMode
    {
        
        _4Bit,
        _8Bit
    }

    class Program
    {
        static void Main(string[] args)
        {
            // Takes an input directory and reads all PNG files in that
            // directory, for each file a reduced, 16 color,
            // pallette is generated and output together with the pixel
            // data as C-Headerfile or rust code.
            // the second parameter contains the output directory,
            // the third parameter contains the output format (either cpp or rust)

            var files = System.IO.Directory.GetFiles(args[0], "*.png");
            List<string> processedFiles = new List<string>();
            foreach(var f in files)
            {
                System.Console.ForegroundColor = ConsoleColor.Green;
                System.Console.WriteLine("Processing: " + f);
                string spriteName = System.IO.Path.GetFileName(f).Replace(".png", "");
                Do4BitDownsample(f, spriteName, args[1], args[2]);
                processedFiles.Add(spriteName);
            }
        }

        static void DoCppOutput(string spriteName, int x, int y, TextWriter target, Color[] palette, List<byte> pixels)
        {
            target.WriteLine("#pragma once");
            target.WriteLine("const char " + spriteName + "_data[] = { ");
            target.Write("\t");

            for (var i = 0; i < pixels.Count; i++)
            {
                var isLast = i == pixels.Count - 1;
                target.Write("0x" + BitConverter.ToString(new[] { pixels[i] }) + (isLast ? "" : ","));
                if ((i+1) % 16 == 0)
                {
                    target.WriteLine();
                    target.Write("\t");
                }
            }
            target.WriteLine("};");

            // We need to write the dimensions of the sprite as well:
            target.WriteLine();
            target.WriteLine($"const int {spriteName}_w = {x};");
            target.WriteLine($"const int {spriteName}_h = {y};");

            // At last, we render the palette for the sprite into the file:
            target.WriteLine();
            target.WriteLine($"uint32_t {spriteName}_pal [{palette.Length}] = " + " { ");
            target.Write("\t");

            for (var i = 0; i < palette.Length; i++)
            {
                var isLast = i == palette.Length - 1;
                var palettEntry = palette[i].ToArgb();
                target.Write($"0x{palettEntry:X4}");
                if(isLast)
                {
                    target.Write("};");
                }
                else
                {
                    target.Write(",");
                }
                if ((i+1) % 8 == 0)
                {
                    target.WriteLine();
                    target.Write("\t");
                }
            }
        }

        static void DoRustOutput(string spriteName, int x, int y, TextWriter target, Color[] palette, List<byte> pixels)
        {
            target.Write($"pub static {spriteName}_data: [u8;{pixels.Count}] = ");
            target.Write("[");
            for (var i = 0; i < pixels.Count; i++)
            {
                var isLast = i == pixels.Count - 1;
                target.Write("0x" + BitConverter.ToString(new[] { pixels[i] }) + (isLast ? "" : ","));
                if ((i + 1) % 16 == 0)
                {
                    target.WriteLine();
                    target.Write("\t");
                }
            }
            target.WriteLine("];");
            target.WriteLine();
            
            target.Write($"pub static {spriteName}_pal: [u32;{palette.Length}]= ");
            target.Write("[");
            for (var i = 0; i < palette.Length; i++)
            {
                var isLast = i == palette.Length - 1;
                var palettEntry = palette[i].ToArgb();
                target.Write($"0x{palettEntry:X4}");
                if (isLast)
                {
                    target.Write("];");
                }
                else
                {
                    target.Write(",");
                }
                if ((i + 1) % 8 == 0)
                {
                    target.WriteLine();
                    target.Write("\t");
                }
            }

        }

        class ColorBin
        {
            public int occurences;
            public Color color;
            public ColorBin(Color c) { color = c; }

            public int clusterIndex;
        }

        private static void Do4BitDownsample(string f, string spriteName, string outputDirectory, string outpoutmode)
        {
            Bitmap image = (Bitmap)System.Drawing.Image.FromFile(f);
            var x = image.Width;
            var y = image.Height;

            if (x % 2 != 0 || y % 2 != 0)
            {
                Console.WriteLine($"Cannot process {f}. dimensions must be multiple of 2");
                return;
            }

            // Create color histogram:
            List<ColorBin> bins = MakeHistogram(image, x, y);
            Color[] palette = doKMeans(bins, 16);
            var pixels = makePixels(image, bins, palette, ColorMode._4Bit);
            StringWriter w = new StringWriter();
            if (outpoutmode.ToLower() == "cpp")
            {
                DoCppOutput(spriteName, x, y, w, palette, pixels);
                File.WriteAllText(outputDirectory + spriteName + ".h", w.ToString());
            }
            else
            {
                DoRustOutput(spriteName, x, y, w, palette, pixels);
                File.WriteAllText(outputDirectory + spriteName + ".rs", w.ToString());
            }
            
        }

        static byte getColorIndex(Color input, Color[] centers, List<ColorBin> bins)
        {
            Color newColor = centers[bins.First(bn => bn.color == input).clusterIndex];
            return (byte)bins.First(bn => bn.color == input).clusterIndex;
        }

        private static List<byte> makePixels(Bitmap image, List<ColorBin> bins, Color[] palette, ColorMode colorMode)
        {
            // Done, now we can render the data into the file:
            List<byte> pixels = new List<byte>();
            StringWriter writer = new StringWriter();


            var output = new List<byte>();

            for (int cy = 0; cy < image.Height; cy++)
            {
                for (int cx = 0; cx < image.Width - 1; cx+= 2)
                {
                    var col0 = getColorIndex(image.GetPixel(cx, cy), palette, bins);
                    var col1 = getColorIndex(image.GetPixel(cx + 1, cy), palette, bins);
                    if(colorMode == ColorMode._4Bit)
                    {
                        var pixelvalue = (byte)(((col0 & 0xF) << 4) + (col1 & 0xF));
                        output.Add(pixelvalue);

                    }
                    else
                    {
                        output.Add(col0);
                        output.Add(col1);
                    }
                }
            }

            return output;
        }

        private static Color[] doKMeans(List<ColorBin> bins, byte numColors)
        {
            // Downsample histo to "numColors" colors:
            // first, we order by occurence (i.e. we find the "numColors" most prevalent colors
            // and then use K-Means over the rest of the bins 
            // to find the best 16 colors:
            var ordered = bins.OrderBy(b => b.occurences).ToList();

            // okay - for the sake of easier debugging, we
            // should reorder the bins such that:
            // - Black and White are always in the same position
            // - The mask color is always in the same position.
            //.. later

            while (ordered.Count < numColors)
                ordered.Add(new ColorBin(Color.Black));

            bool done = false;
            Color[] centers = new Color[numColors];
            for (int i = 0; i < numColors; i++)
                centers[i] = ordered[i].color;

            while (!done)
            {
                done = true;
                foreach (var bin in bins)
                {
                    int newCluster = BestClusterIndex(bin.color, centers);

                    if (newCluster != bin.clusterIndex)
                        done = false;
                    bin.clusterIndex = newCluster;
                }

                // Recalculate cluster centers
                for (int i = 0; i < centers.Length; i++)
                {
                    var theCenter = centers[i];

                    // Now, we lock White and Black - these should not shift around
                    // ever.
                    // --> Note, that we need a way of masking pixels as well, and all
                    // pixels containing the mask color should be hardlocked as well
                    // (also they should not be considered for the K-Means bit.
                    if (theCenter.R == 255 && theCenter.G == 255 && theCenter.B == 255)
                        continue;
                    if (theCenter.R == 0 && theCenter.G == 0 && theCenter.B == 0)
                        continue;
                    // Mask Color - is bright pink
                    if (theCenter.R == 255 && theCenter.G == 0 && theCenter.B == 255)
                        continue;

                    var members = bins.Where(bin => bin.clusterIndex == i).ToArray();

                    //calc new average color from members:
                    var r = members.Sum(m => m.color.R) / members.Length;
                    var g = members.Sum(m => m.color.G) / members.Length;
                    var b = members.Sum(m => m.color.B) / members.Length;
                    centers[i] = Color.FromArgb(255, r, g, b);
                }

            }

            return centers;
        }

        private static List<ColorBin> MakeHistogram(Bitmap image, int x, int y)
        {
            var bins = new List<ColorBin>();
            for (int cy = 0; cy < y; cy++)
            {
                for (int cx = 0; cx < x; cx++)
                {
                    Color c = image.GetPixel(cx, cy);

                    ColorBin theBin = bins.FirstOrDefault(b => b.color == c) ?? new ColorBin(c);
                    theBin.occurences++;

                    if (theBin.occurences == 1)
                        bins.Add(theBin);

                }
            }
            return bins;
        }

        static int BestClusterIndex(Color target, Color[] clusters)
        {
            // order clusters by their distance to the target color.
            
            var ordered = clusters.OrderBy(x =>
            {
                // Hack for masks: the mask color is bright pink,
                // and we don't want any color to be accidently binned
                // into the mask, therefore, we check if x is the mask.
                // if so, and target != mask, we return an insanely large
                // number, thus making the mask the "farthest" away color
                // resulting in it never being chosen unless the mask color
                // is actually intended.

                if(x.R == 255 && x.B == 255 & x.G == 0) // x == mask!
                {
                    if(target.R == 255 && target.B == 255 && target.G == 0)   // target is mask as well!
                    {
                        return 0;
                    }
                    // current color is mask, but the target color is not
                    // mask -> never choose this.
                    return int.MaxValue;
                }

                int dstR = x.R - target.R;
                int dstG = x.G - target.G;
                int dstB = x.B - target.B;
                return (dstR * dstR) + (dstG * dstG) + (dstB * dstB);
            }).ToArray();

            // take the one with the least distance (i.e. the first in the range!)
            var theColor = ordered.First();
            for(int i = 0; i < clusters.Length; i++)
            {
                if (theColor == clusters[i])
                    return i;
            }
            return 0;
        }
    }
}
