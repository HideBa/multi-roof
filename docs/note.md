# Note for the task

## What is the definition of LoD1.2 and LoD2.2?
>
> * LoD1.2: LOD 1.2 have the requirement that smaller building parts and extensions should be acquired (e.g. alcoves), and are extruded to a single height. (Biljecki, 2014)
>
> * LoD2.2: LOD2.2 follows the requirements of LOD2.0 and LOD2.1, with the addition of roof superstructures (larger than 2 m and 2 m2) to be acquired. (Biljecki, 2014)
>
## Why do we need lower LoD models when we have higher LoD models?
>
> LOD1 models provide a relatively high information content and usability comparing to their geometric detail (Henn et al., 2012; Hofierka and Zlocha, 2012). For instance, they may be used for shadowing simulations (Strzalka et al., 2012; Alam et al., 2013; Li et al., 2015), estimation of noise pollution (Stoter et al., 2008), energy demand estimation (Strzalka et al., 2011; Bahu et al., 2013), simulating floods (Varduhn et al., 2015), analysing wind comfort (Amorim et al., 2012), and visualisation (Gesquière and Manin, 2012).
>
## What properties should be kept in the lower LoD model?

According to Biljecki et al. (2014), they have evaluated these properties for the lower LoD model:

* Average triangle count per building
* Total surface area
* Area of the WallSurface
* Volume of the corresponding solid
* Size in memory for each representation

## Method

To simplify the problem for now, we make some assumptions:

* Geometry is topologically correct, such as:
  * No intersecting or self-intersecting faces
  * Watertight
* Vertices orientation follows the right-hand rule (Normal vector points outwards)
* We aim to create an LoD1.2 model which has a good balance of the properties mentioned above

<!--
* try to keep the same volume as ?.2 model as possible. As Filip 2014 paper says, even in the same level of detail of CityGML, the volume of the model can be different. such as lod2.2 and lod2.1. That's why Filip degines finer level of detail as the base model. -->

## Steps

1. Read input model as OBJ file.

    * Assume the input model is already triangulated
    * We don't support all specifications of the OBJ file such as material

2. Correct the invalid geometry or inconsistent geometry in the input model.

    * While this is very important, we deliberately ignore it in this task because it's not the focus

3. Label the geometry with GroundSurface, WallSurface, and RoofSurface.

    * Calculate the normal vectors of the faces
    * Label the faces with GroundSurface, WallSurface, and RoofSurface based on the normal vectors:
      * Calculate the lowest z value of the faces
      * For each surface:
        * If the normal vector is close to 0 degrees against the z-axis and the z value is within 1.5 m of the lowest z value, it is a GroundSurface
        * If the normal vector is close to 90 degrees against the z-axis, it is a WallSurface
        * Otherwise, it is a RoofSurface

4. Decide the height of the building.

    * Calculate the area of each roof surface
    * Calculate the height (max z value - min z value) of each roof surface
    * Calculate the height of the entire roof surface as the weighted average of each roof surface's height, with weight being the area of each roof surface
    * Remove all roof surfaces
    * Remove all wall surfaces
    * Identify GroundSurface triangles that don't have three adjacent triangles (As the original model is triangulated, it should have three adjacent triangles by default)

5. Write the output model as OBJ file.

## Discussion

### How to label the geometry with GroundSurface, WallSurface, and RoofSurface?

#### Option 1

##### Steps

* If the normal vector is close to 90 degrees against the z-axis, it is a WallSurface
* If the normal vector is positive against the z-axis, it is a RoofSurface
* If the normal vector is negative against the z-axis, it is a GroundSurface

##### Pros and Cons

* Pros:
  * Simple and fast
* Cons:
  * Models don't always have consistent surface orientation (even ground surfaces might point upwards)
  * Not robust for outliers such as underground parts

#### Option 2

##### Steps

* Mark the surface which has the lower height as the GroundSurface. Then walk adjacent surfaces:
  * If the adjacent surface's normal vector is close to 0 degrees against the z-axis, it is a GroundSurface
  * If the normal vector is close to 90 degrees against the z-axis, it is a WallSurface
  * Otherwise, it is a RoofSurface

##### Pros and Cons

TODO: write pros and cons

#### Option 3 (I've used this)

##### Steps

* Mark the surface which has the lowest z value as the GroundSurface
* Calculate the height (max z value - min z value) of the surface
* If the adjacent surface's normal vector is close to 0 degrees against the z-axis, it is a GroundSurface
* If the normal vector is close to 90 degrees against the z-axis, it is a WallSurface
* Otherwise, it is a RoofSurface

##### Pros and Cons

TODO: write pros and cons

### What height should we use for the lower LoD model?

#### Option 1 (I've used this)

I've used the weighted average of roof surface heights for the lower LoD model.
The height of the roof surface is calculated by the following formula:

`h = (max(z) - min(z)) * ROOF_HEIGHT_PERCENTILE * area / sum(area)`

* Pros:
  * Simple and fast
  * The model doesn't need to be water-tight
  * Robust against outliers such as antennas, etc.
* Cons:
  * Volume and total surface area might differ from the original model
  * Higher time complexity than simply calculating the mean height of roof surfaces

#### Option 2

Calculate the roof height that maintains the same volume as the original model.

* Pros:
  * By preserving volume, it's more useful for specific simulations such as energy demand estimation
* Cons:
  * The original model needs to be water-tight to calculate volume. We would need to make the model water-tight first, perhaps by snapping vertices

#### Option 3

Simply calculate the mean or a given threshold height, such as 70% of the distance between the roof's top and bottom.

* Pros:
  * Simple and fast
* Cons:
  * Not robust against outliers such as antennas, etc.
  * Won't maintain the same volume as the original model

### Why I chose Option 1

* Even though the example model is not water-tight, I chose this option because it's simple and fast
* It simplifies fragmented roofs by clustering and taking only the major part
* It removes outliers such as antennas, etc.

#### Acknowledgement

* We need to address invalid geometry or inconsistent geometry in the input model
* There are exceptional shapes in the input model, such as buildings with holes in the middle
* The assumption that volume remains the same is not always valid
* This approach has only been tested with building examples, not with other types of City Objects
