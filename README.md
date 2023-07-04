# Resources

 - [Overpass](https://overpass-turbo.eu/) - Query and select data, and export it to a number of different files
 - [OSM Export](https://www.openstreetmap.org/export#map=19/-37.79574/144.92911) - Easily grab and export a bounding box on a map

# OSM Queries

## Get residential roads within the bounding box

```osm
[timeout:25][bbox:-37.796223, 144.928243, -37.796100, 144.928732];

(
  way["highway"="residential"];
);

out body;
>;
out skel qt;
```

# Other Notes

## Getting data into PBF format

1. Download desired data from one of the links above (export raw data from Overpass API)
2. Download and compile `osmconvert`: `http get http://m.m.i24.cc/osmconvert.c | cc -x c - -lz -O3 -o osmconvert`
3. Run the following command to perform the conversion: `cat [downloaded file] | ./osmconvert - --out-pbf -o=[PBF output file]`

