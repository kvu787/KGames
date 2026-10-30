# PaintSettings

Reads and writes [SimplePaint](../SimplePaint/README.md) materials in programs' JSON files, in
the format of the settings of Veehiicuul's C++ scene, so that a material can be copied between
programs, like from SimplePaintDemo's `Paint.json` into another program's `Settings.json`:

```json
{
    "BaseColor": [0.107, 0.223, 0.578],
    "Brightness": 0.5,
    "Shift": 0.0,
    "RotationDegrees": 0.0,
    "DarkPoint": 0.0,
    "LightPoint": 1.0
}
```

A member that's missing keeps its default, and a value that SimplePaint rejects is an error.
SimplePaint itself leaves serialization to its hosts.
