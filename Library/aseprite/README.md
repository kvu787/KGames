# Aseprite

Reads [Aseprite](https://www.aseprite.org/)'s `.aseprite` files: the first frame of an RGBA
sprite, as its canvas shows it, with its visible layers flattened, or one of its layers on its own.
It has no dependencies: it decompresses the cels' zlib streams itself.

| Function or type             | What it is                                                             |
| ---------------------------- | ---------------------------------------------------------------------- |
| `load`, `read`               | The first frame of a file, from its path or its bytes, as a `Canvas`   |
| `load_layer`, `read_layer`   | One layer of the first frame, by its name, on its own, as a `Canvas`   |
| `Canvas`                     | Pixels as red, green, blue and alpha, not premultiplied, top row first |
| `blend_normal`               | Aseprite's Normal blend mode, which flattens the layers                |
| `zlib_decompress`, `adler32` | Decompresses a zlib stream of DEFLATE data, and checks its checksum    |

## What it reads

The [file format's specification](https://github.com/aseprite/aseprite/blob/main/docs/ase-file-specs.md)
describes every chunk. Of them, the canvas needs only the first frame's layers and cels:

- A layer shows if it's visible, and so is each group it's in. Reference layers don't show, as
  Aseprite doesn't export them.
- Cels go from the bottom layer up, each in the order of its layer's index plus its z-index.
- Each cel blends over what's below it with the Normal blend mode, at its opacity times its
  layer's, when the file's header says that layers' opacity is valid.
- Raw cels and zlib-compressed ones are read. Parts of a cel off the canvas are left out.

A layer on its own is its cel on a transparent canvas, at its opacity, as the canvas would show it
if it were the only visible layer, whether it's visible or not. It has to be the only layer with its
name, and a layer of pixels, not a group or a tilemap.

It fails, saying why, for what it doesn't read, rather than show a canvas that Aseprite wouldn't:
sprites in Indexed or Grayscale color mode, tilemap layers that show, layers that show with a
blend mode other than Normal, and files that are cut short or broken.

Tests are in [Test/aseprite](../../Test/aseprite).
