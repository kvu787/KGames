# Design doc

Question: figure out how to maintain a consistent representation for visual objects (car, cone, house, etc.) throughout the workflow

## Blender

- Model mesh in Blender
- Types of meshes
  - Single mesh object:
    - This is when you export a single mesh object in blender
    - To assign different colors to different faces, you must use blender material slots
    - In blender, it seems like you can have zero material slots, so I'll need to test glb this exports to
    - This can be made by joining multiple separate mesh objects together
      - Alternatively, you can keep the objects separate but under a single root
  - Single root, multiple objects
    - Can contain empty or mesh object types
  - In any case, the mesh must have a single "pivot point" to make it easy to work with on the Veehiicuul side
    - This means if you have multiple mesh objects you want to export, you need to parent them to an empty first
- When an object gets in the veehiicuul program boundary, edits should be limited to this:
  - Transform: Position, scale, rotation
  - Setting colors
  - Veehiicuul shouldn't support general mesh editing like adjusting vertex positions, editing mesh object hierarchy, or modeling new meshes
- To make adjustments easy, you should be able to color in a mesh on the Blender side or the Veehiicuul side

## Veehiicuul

- Hexel terrain editor
- Track builder
- Vehicle footprint creator
- Player
