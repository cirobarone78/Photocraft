# Global Perspective Warp V3

BCS local extension of PhotoCraft 0.8.2. This tool is distinct from the existing multi-plane **Perspective Warp**. It is not a DLL plug-in and does not replace the legacy algorithm.

## What the guides mean

Use Edit > Global Perspective Warp (French: Edition > Deformation de perspective globale). Draw independent quadrilaterals on the four corners of rectangular doors, windows or frames. Corners are ordered top-left, top-right, bottom-right, bottom-left in the source photograph. Guides do not need to touch, overlap or share edges. They are observations of two architectural directions, NOT separate image regions.

Click **Rectify globally**. A single projective homography is estimated jointly from compatible guides and applied to the entire active raster photo layer. No local maps are blended and no image regions are stitched. Other document layers are not transformed. The legacy **Link Edges** button is neither needed nor present in this mode.

In Layout, drag to create a rectangular guide, then move its four handles onto the photographed corners. Click inside a guide or use its number in the toolbar to select it. Shift-drag allows a new guide inside an existing one. **Enabled**, **Delete guide** and the guide weight in **Options** let you correct observations without changing other guides. In Result view, guide handles show the transformed source positions; they cannot deform windows independently. Return to **Edit guides** to adjust the source observations, then rectify again. Changing guides or options invalidates the old solution.

The checkmark / Enter commits one undoable engine operation. Escape / Cancel discards the preview without editing pixels. A failed solve or commit preserves the editable session. A document switch or external document edit refuses a stale commit: cancel and reopen on the intended source document.

## Geometric model and limits

The solver uses one global 3x3 matrix and fits the horizontal and vertical edge families across the image. Coordinates are centered and isotropically normalized for numerical stability. Deterministic robust hypotheses and consensus refinement reduce the effect of inconsistent guides. Both direction families must ultimately share a strict majority of compatible enabled guides. A guide is compatible when all its transformed edges are within 3 degrees of their intended axis. Rejected guides remain visible in the result, with warning coloring and their IDs in Options and machine-readable diagnostics. Their pixels are still transformed by the SAME matrix.

A single valid quadrilateral can provide a solution, but has no independent consistency check. Nearly singular, mirrored or excessive transforms are rejected rather than forced. Horizontal/vertical labels assume upright architectural observations; severe orientation ambiguity may require rotating the photo first. Different facade directions, lens distortion, inaccurate corners and genuinely non-rectangular openings may be incompatible with one homography. The tool reports that limitation; it never adds local folds to satisfy conflicting observations.

**Unknown rectangular dimensions do not determine true physical aspect ratios.** The default preserves apparent central direction scales. `aspect` is an explicit global width/height adjustment, not a recovered measurement. A uniform output scale and translation fit the whole transformed footprint inside the original raster bounds by default (`fitCanvas: true`). This retains the complete image with transparent margins rather than cropping. Turning it off permits pixels outside the document canvas; subsequent flat export may crop to that canvas. There is no generative fill.

Safety is checked over the complete source rectangle, not only at guide corners: the projective denominator must remain positive and bounded away from zero at all four corners; its affine form proves that there is no horizon crossing inside. A nonsingular matrix with positive determinant then has positive Jacobian everywhere on that domain. This is an analytic no-fold/injectivity guard, not a claim based solely on sampling mesh cells. Output expansion, coordinates and pixel counts are also bounded.

Limits: at most 128 guides, finite source guide coordinates (including positions outside the photo, absolute coordinate limit 1,000,000), positive weights 0.05..20, aspect 0.25..4, at most 32768 pixels on one image side, and a final raster allocation budget of 67,108,864 output pixels including filter support. Nonfinite, duplicate-ID, degenerate and non-clockwise guides are rejected. The V3 initial scope is an **unmasked raster photo layer**, including Background (converted in the same undoable edit). Selections, layer masks, vector masks and Smart Objects are explicitly rejected; use a rasterized/flattened COPY rather than silently dropping those features. The original file is never automatically overwritten.

Final rendering uses the existing inverse projective resampler and retains the image's supported color model/sample depth, adding transparency where needed. Preview geometry uses the very same matrix on a 128x128 display mesh with a bounded texture. Texture filtering and reduced display resolution can differ from final pixels; the geometric solution does not. This is not a promise of bit-identical preview colors at different zooms.

## Structured AI interface

Command ID: `edit.perspectiveWarpGlobal`. All coordinates are source document pixels.

A guide:

```json
{"id":1,"corners":[[25,20],[100,16],[109,105],[21,110]],"enabled":true,"weight":1}
```

`id` and `corners` are required. `enabled` defaults to true and `weight` to 1. IDs must be unique. No destination quadrilateral is accepted.

Direct engine invocation:

```json
{"method":"engine.execute","params":{"command":"edit.perspectiveWarpGlobal","params":{"guides":[{"id":1,"corners":[[25,20],[100,16],[109,105],[21,110]]}],"options":{"fitCanvas":true,"aspect":1},"solveOnly":true}}}
```

Direct parameters are `guides`, `options`, optional `layer`, optional `document`, `solveOnly` (default false), and `interpolation` (`bicubic`, `bilinear`, `nearest`; default bicubic). Unknown keys and wrong types are rejected. `solveOnly: true` computes/validates diagnostics without changing document or history. `solveOnly: false` computes and applies one operation. An identity result returns `changed: false` and adds no history step. Explicit document IDs protect against acting on the wrong tab. Selection-based partial warps are not supported.

Interactive workflow, each item sent separately through the existing authenticated control transport:

```json
["ui.menu.invoke",{"id":"edit.perspectiveWarpGlobal"}]
["ui.menu.invoke",{"id":"edit.perspectiveWarpGlobal","params":{"ui":{"guides":[{"id":1,"corners":[[25,20],[100,16],[109,105],[21,110]]}],"solve":true}}}]
["ui.inspect",{}]
["ui.menu.invoke",{"id":"edit.perspectiveWarpGlobal","params":{"ui":{"commit":true}}}]
```

Interactive `params` may contain ONLY `ui`; use `engine.execute` for direct engine data. UI fields: `guides` (replace complete list), `options` (replace options object with documented defaults), `mode` (`layout|result`), `selectedGuide` (existing ID), `solve`, `commit`, `cancel`. `result` requests a current solve. `layout` changes the view without changing geometry. Replacement data and a requested solve are validated atomically before changing the old session. Optional `null` guide/options/selection/mode fields mean omitted; booleans must be booleans. Commit and cancel must be standalone actions, not combined with replacements or solve. Commit requires a solved, unchanged source session. An absent/invalid solution is never interpreted as success.

`ui.inspect` exposes `distort.globalPerspective` (alongside legacy `distort.perspective`). It includes `version`, `document`, `layer`, `mode`, source `guides`, `selectedGuide`, `options`, `solved`, `dirty`, `scope`, and `solution`. The solution includes:

- `model: singleGlobalHomography`, row-major `matrix[9]`, `sourceBounds`, `outputBounds`;
- `usedGuideIds`, `rejectedGuideIds`, and per-guide `enabled`, `used`, `maxErrorDeg`, `transformedCorners`;
- `maxUsedErrorDeg`, `minDenominator`, `minJacobian`, `aspectAmbiguous`, and `warnings`.

Do not equate `solved` with human visual acceptance, metric calibration or compatibility of rejected guides. Both UI and engine call the same solver.

## Reproducible local diagnostic

The `photocraft-algo` example `global_guides` reads a local JSON fixture with `imageSize`, `guides` and optional `options`, and creates a new diagnostics JSON without modifying an image:

```text
cargo run --locked -p photocraft-algo --example global_guides -- fixture.json new-output.json
```

The example refuses to overwrite an existing output. Private photographs and screenshot-derived guide coordinates stay outside the Git tree and distributable demo package. The bundled demo is an original synthetic facade, not a private photograph.

## Mathematical background

The implementation is clean-room Rust based on projective geometry and independently derived line constraints, not copied third-party implementation code. Background: D. Liebowitz and A. Zisserman, *Metric Rectification for Perspective Images of Planes*, CVPR 1998; Oxford Visual Geometry Group publication page and planar-rectification notes. A physical metric upgrade requires sufficient independent metric information; the default here explicitly does not claim that upgrade.

## Guide utilisateur francais

1. Extraire le ZIP V3 dans un nouveau dossier et conserver `portable.txt` a cote de `photocraft.exe`.
2. Lancer cet executable, puis ouvrir une copie de la photo.
3. Choisir **Edition > Deformation de perspective globale**. Ne pas choisir l'ancien outil multi-plans.
4. Tracer un cadre par porte/fenetre, puis ajuster les quatre coins. Les cadres restent independants.
5. Cliquer sur **Redresser globalement**. Le bouton de liaison des aretes n'est pas utilise.
6. Examiner les murs entre les cadres. Un repere en couleur d'avertissement est incompatible : le corriger ou le desactiver.
7. Dans **Options**, conserver **Conserver toute la photo** pour eviter le recadrage implicite. **Proportions globales** corrige la largeur/hauteur de toute l'image, jamais une fenetre seule.
8. Valider avec la coche ou Entree. Annuler avec Echap ou restaurer avec Undo. Enregistrer dans un nouveau fichier.

Les limites de plans 3D differents et de proportions reelles restent explicites. Une photo globalement coherente prime sur la rectification forcee de chaque ouverture.

### Precision guide editing

- **Ctrl + drag a corner**: constrain the movement to the horizontal or vertical axis with the strongest initial drag motion. Releasing Ctrl restores free movement. The other corners remain stationary.
- **Hover an edge**: the cursor changes to a move icon. Drag the edge to translate **both of its endpoint corners together**, without moving the opposite side. Ctrl constrains the edge translation to one axis as well.
- Corner handles take priority over edges at intersections. These edits only update guide geometry; global rectification is recomputed when **Rectify globally** is pressed. Result preview does not permit local warping.

- **Beyond the photo**: drag a guide corner or edge into the pasteboard outside the image to align extended architectural lines. The guide may extend beyond the pixels; only the photo is transformed. Safety validation still rejects self-intersections, extreme coordinates, or a projective map that becomes unsafe.
