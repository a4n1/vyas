struct Camera {
    inverse_view_proj: mat4x4<f32>,
    position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct RenderConfig {
    brickmap_size: u32,
    brickgrid_size: u32,
    fill_color: vec3f,
};

@group(0) @binding(1)
var<uniform> config: RenderConfig;

@group(0) @binding(2)
var<storage, read> color_palette: array<u32, 256>;

@group(0) @binding(3)
var<storage, read_write> brickgrid: array<atomic<u32>>;

struct Feedback {
    count: atomic<u32>,
    data: array<u32, 4096>,
}

@group(0) @binding(4)
var<storage, read_write> feedback: Feedback;

struct Brickmap {
    solid_mask: array<u32, 16>,
    color_index: u32,
};

@group(0) @binding(5)
var<storage, read> brickmaps: array<Brickmap>;

@group(0) @binding(6)
var<storage, read> colors: array<u32>;

@group(0) @binding(7)
var output_texture: texture_storage_2d<rgba16float, write>;

@group(0) @binding(8)
var output_sampled: texture_2d<f32>;

@group(0) @binding(9)
var output_sampler: sampler;

@compute @workgroup_size(8, 8) 
fn raytrace(@builtin(global_invocation_id) id: vec3u) {
    const directionEpsilon = vec3f(0.000001);

    let brickgridSize = i32(config.brickgrid_size);
    let brickmapSize = f32(config.brickmap_size);
    let gridRange = (brickgridSize - 1) / 2;

    let dimensions = textureDimensions(output_texture);

    if (id.x >= dimensions.x || id.y >= dimensions.y) {
        return;
    }

    let position = id.xy;
    textureStore(output_texture, position, vec4f(config.fill_color, 1.0));

    let rayOrigin = camera.position.xyz;
    let rayDirection = getRayDirection(vec2f(position), vec2f(dimensions));

    let brickmapRayOrigin = rayOrigin / brickmapSize;
    let brickmapPosition = vec3i(floor(brickmapRayOrigin));
    let traversalDirection = select(
        directionEpsilon,
        rayDirection,
        abs(rayDirection) > directionEpsilon,
    );

    let stepDirection = vec3i(sign(traversalDirection));
    let boundarySpacing = 1.0 / abs(traversalDirection);

    var stepPosition = brickmapPosition;
    var boundaryDistance = (
        vec3f(stepDirection) * (vec3f(stepPosition) - brickmapRayOrigin)
        + 0.5 * vec3f(stepDirection)
        + 0.5
    ) * boundarySpacing;
    var rayEntryDistance = 0.0;

    for (var i = 0; i < brickgridSize * 3; i++) {
        let gridPosition = vec3i(
            stepPosition.x - brickmapPosition.x + gridRange,
            brickmapPosition.y - stepPosition.y + gridRange,
            brickmapPosition.z - stepPosition.z + gridRange,
        );

        if any(gridPosition < vec3i(0)) || any(gridPosition >= vec3i(brickgridSize)) {
            break;
        }

        let index = getIndex(gridPosition, brickgridSize);
        let brickgridCell = atomicLoad(&brickgrid[index]);

        if brickgridCell != 0 {
            const unloadedBit = 1u << 24u;
            const requestedBit = 1u << 25u;

            let isLoaded = brickgridCell != 0u && (brickgridCell & unloadedBit) == 0u;
            let isUnloaded = (brickgridCell & unloadedBit) != 0u;

            if isLoaded {
                let brickmap = brickmaps[brickgridCell - 1u];
                let localRayOrigin = clamp(
                    rayOrigin
                        + rayDirection * rayEntryDistance
                        - vec3f(stepPosition) * brickmapSize,
                    vec3f(0.0),
                    vec3f(brickmapSize - 0.000001),
                );

                if raytraceBrickmap(position, localRayOrigin, traversalDirection, brickmap) {
                    return;
                }
            } else if isUnloaded {
                let previous = atomicOr(&brickgrid[index], requestedBit);

                if (previous & requestedBit) == 0u {
                    let i = atomicAdd(&feedback.count, 1u);

                    if i < 4096u {
                        feedback.data[i] = index;
                    }
                }

                return;
            }
        }

        rayEntryDistance = min(
            min(boundaryDistance.x, boundaryDistance.y),
            boundaryDistance.z,
        ) * brickmapSize;

        let mask = boundaryDistance <= min(boundaryDistance.yzx, boundaryDistance.zxy);

        boundaryDistance += vec3f(mask) * boundarySpacing;
        stepPosition += vec3i(mask) * stepDirection;
    }

    textureStore(output_texture, position, vec4f(config.fill_color, 1.0));
}

fn raytraceBrickmap(
    position: vec2u,
    localRayOrigin: vec3f,
    rayDirection: vec3f,
    brickmap: Brickmap,
) -> bool {
    let brickmapSize = i32(config.brickmap_size);

    var mapPos = vec3i(floor(localRayOrigin));

    let deltaDist = 1.0 / abs(rayDirection);
    let rayStep = vec3i(sign(rayDirection));

    var sideDist = (
        vec3f(rayStep) * (vec3f(mapPos) - localRayOrigin)
        + 0.5 * vec3f(rayStep)
        + 0.5
    ) * deltaDist;

    for (var i = 0; i < brickmapSize * 3; i++) {
        if any(mapPos < vec3i(0)) || any(mapPos >= vec3i(brickmapSize)) {
            return false;
        }

        let voxelIndex = getIndex(mapPos, brickmapSize);
        let wordIndex = voxelIndex / 32u;
        let bitIndex = voxelIndex % 32u;

        let isSolid = (brickmap.solid_mask[wordIndex] & (1u << bitIndex)) != 0u;

        if isSolid {
            var colorIndex = 0u;

            for (var j = 0u; j < wordIndex; j++) {
                colorIndex += countOneBits(brickmap.solid_mask[j]);
            }

            let precedingBits = (1u << bitIndex) - 1u;
            colorIndex += countOneBits(brickmap.solid_mask[wordIndex] & precedingBits);

            let colorByteIndex = brickmap.color_index + colorIndex;
            let packedColorIndices = colors[colorByteIndex / 4u];
            let paletteIndex = (packedColorIndices >> ((colorByteIndex % 4u) * 8u)) & 0xffu;
            let color = color_palette[paletteIndex];

            textureStore(output_texture, position, vec4f(unpackColor(color), 1.0));
            return true;
        }

        let mask = sideDist <= min(sideDist.yzx, sideDist.zxy);

        sideDist += vec3f(mask) * deltaDist;
        mapPos += vec3i(mask) * rayStep;
    }

    return false;
}

fn unpackColor(color: u32) -> vec3f {
    let unpackedColor = vec3f(
        f32(color & 0xffu),
        f32((color >> 8u) & 0xffu),
        f32((color >> 16u) & 0xffu),
    ) / 255.0;

    return srgbToLinear(unpackedColor);
}

fn srgbToLinear(color: vec3f) -> vec3f {
    return select(
        color / 12.92,
        pow((color + 0.055) / 1.055, vec3f(2.4)),
        color > vec3f(0.04045),
    );
}

fn getRayDirection(position: vec2f, dimensions: vec2f) -> vec3f {
    let uv = (position + vec2f(0.5)) / dimensions;

    let ndc = vec2f(
        uv.x * 2.0 - 1.0,
        1.0 - uv.y * 2.0
    );

    let worldH = camera.inverse_view_proj * vec4f(ndc, 1.0, 1.0);
    let world = worldH.xyz / worldH.w;

    return normalize(world - camera.position.xyz);
}

fn getIndex(position: vec3i, size: i32) -> u32 {
    return u32(position.x + position.y * size + position.z * size * size);
}

struct Vertex {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> Vertex {
    let positions = array(
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0),
    );

    let position = positions[idx];

    var output: Vertex;
    output.position = vec4(position, 0.0, 1.0);
    output.uv = vec2(position.x * 0.5 + 0.5, 0.5 - position.y * 0.5);
    return output;
}

@fragment
fn fs_main(input: Vertex) -> @location(0) vec4<f32> {
    return textureSample(output_sampled, output_sampler, input.uv);
}
