struct Camera {
    inverse_view_proj: mat4x4<f32>,
    position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct RenderConfig {
    brickmap_size: u32,
    brickgrid_size: u32,
    crt: u32,
    fallback_brickmap_size: u32,
    fill_color: vec3f,
    time: f32,
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

struct FallbackBrickmap {
    solid_mask: u32,
    colors: array<u32, 2>,
    padding: u32,
};

@group(0) @binding(6)
var<storage, read> fallback_brickmaps: array<FallbackBrickmap>;

@group(0) @binding(7)
var<storage, read> colors: array<u32>;

struct Lights {
    ambient_light: vec4f,
    directional_light: vec4f,
    direction: vec4f,
};

@group(0) @binding(8)
var<uniform> lights: Lights;

@group(0) @binding(9)
var base_color_texture: texture_storage_2d<rgba16float, write>;

@group(0) @binding(10)
var base_color_sampled: texture_2d<f32>;

@group(0) @binding(11)
var surface_data_texture: texture_storage_2d<rgba8uint, write>;

@group(0) @binding(12)
var surface_data_sampled: texture_2d<u32>;

@group(0) @binding(13)
var base_color_sampler: sampler;

@compute @workgroup_size(8, 8) 
fn raytrace(@builtin(global_invocation_id) id: vec3u) {
    const directionEpsilon = vec3f(0.000001);

    let brickgridSize = i32(config.brickgrid_size);
    let brickmapSize = f32(config.brickmap_size);
    let gridRange = (brickgridSize - 1) / 2;

    let dimensions = textureDimensions(base_color_texture);

    if (id.x >= dimensions.x || id.y >= dimensions.y) {
        return;
    }

    let position = id.xy;
    textureStore(base_color_texture, position, vec4f(config.fill_color, 1.0));
    textureStore(surface_data_texture, position, vec4u(0u));

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
    var entryFaceId = 0u;

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

            let localRayOrigin = clamp(
                rayOrigin
                    + rayDirection * rayEntryDistance
                    - vec3f(stepPosition) * brickmapSize,
                vec3f(0.0),
                vec3f(brickmapSize - 0.000001),
            );

            if isLoaded {
                let brickmap = brickmaps[brickgridCell - 1u];

                if raytraceBrickmap(
                    position,
                    localRayOrigin,
                    traversalDirection,
                    brickmap,
                    entryFaceId,
                    vec3f(stepPosition) * brickmapSize,
                ) {
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

                if raytraceFallbackBrickmap(
                    position,
                    localRayOrigin,
                    traversalDirection,
                    fallback_brickmaps[index],
                    entryFaceId,
                    vec3f(stepPosition) * brickmapSize,
                ) {
                    return;
                }
            }
        }

        rayEntryDistance = min(
            min(boundaryDistance.x, boundaryDistance.y),
            boundaryDistance.z,
        ) * brickmapSize;

        let mask = boundaryDistance <= min(boundaryDistance.yzx, boundaryDistance.zxy);
        entryFaceId = faceIdFromNormal(-vec3i(mask) * stepDirection);

        boundaryDistance += vec3f(mask) * boundarySpacing;
        stepPosition += vec3i(mask) * stepDirection;
    }

    textureStore(base_color_texture, position, vec4f(skyColor(rayOrigin, rayDirection), 1.0));
}

fn skyColor(rayOrigin: vec3f, rayDirection: vec3f) -> vec3f {
    const cloudHeight: f32 = 1000.0;
    const cloudScale: f32 = 0.002;
    const cloudOctaves: u32 = 4u;
    const cloudBaseWind: vec2f = vec2f(0.05, 0.025);
    const cloudWarpWind: vec2f = vec2f(0.08, -0.02);
    const cloudWarpOffset: vec2f = vec2f(5.2, 1.3);
    const cloudWarpStrength: f32 = 1.0;
    const cloudDensityThreshold: f32 = 0.6;
    const cloudEdgeSoftness: f32 = 0.2;
    const cloudHorizonFade: f32 = 0.5;
    const cloudColor: vec3f = vec3f(1.0);
    const cloudOpacity: vec3f = vec3f(0.8);

    let horizon = vec3f(0.58, 0.78, 0.96);
    let zenith = vec3f(0.18, 0.43, 0.82);
    let sky = mix(horizon, zenith, clamp(rayDirection.y * 2.0, 0.0, 1.0));

    if rayDirection.y <= 0.0 {
        return sky;
    }

    let distance = (cloudHeight - rayOrigin.y) / max(rayDirection.y, 0.01);
    let p = (rayOrigin.xz + rayDirection.xz * distance) * cloudScale;
    let baseWind = config.time * cloudBaseWind;
    let warpWind = config.time * cloudWarpWind;
    let q = vec2f(
        fbm(p + warpWind, cloudOctaves),
        fbm(p + warpWind + cloudWarpOffset, cloudOctaves),
    );
    let density = fbm(p + baseWind + q * cloudWarpStrength, cloudOctaves);
    let cloudMask = smoothstep(
        cloudDensityThreshold,
        cloudDensityThreshold + cloudEdgeSoftness,
        density,
    ) * smoothstep(0.0, cloudHorizonFade, rayDirection.y);

    return mix(sky, cloudColor, cloudMask * cloudOpacity);
}

fn hash_2d(p: vec2f) -> f32 {
    let n = dot(p, vec2f(127.1, 311.7));
    return fract(sin(n) * 43758.5453123);
}

fn noise_2d(p: vec2f) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let a = hash_2d(i);
    let b = hash_2d(i + vec2f(1.0, 0.0));
    let c = hash_2d(i + vec2f(0.0, 1.0));
    let d = hash_2d(i + vec2f(1.0, 1.0));

    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2f, octaves: u32) -> f32 {
    var value = 0.0;
    var amp = 0.5;
    var freq = 1.0;

    for (var i = 0u; i < octaves; i++) {
        value += amp * noise_2d(p * freq);
        freq *= 2.0;
        amp *= 0.5;
    }
    return value;
}

fn raytraceBrickmap(
    position: vec2u,
    localRayOrigin: vec3f,
    rayDirection: vec3f,
    brickmap: Brickmap,
    entryFaceId: u32,
    brickmapOrigin: vec3f,
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
    var faceId = entryFaceId;

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

            let shadowPosition = voxelFaceCenter(brickmapOrigin, mapPos, 1.0, faceId);
            writeSurfaceHit(position, color, faceId, shadowPosition);

            return true;
        }

        let mask = sideDist <= min(sideDist.yzx, sideDist.zxy);
        faceId = faceIdFromNormal(-vec3i(mask) * rayStep);

        sideDist += vec3f(mask) * deltaDist;
        mapPos += vec3i(mask) * rayStep;
    }

    return false;
}

fn raytraceFallbackBrickmap(
    position: vec2u,
    localRayOrigin: vec3f,
    rayDirection: vec3f,
    brickmap: FallbackBrickmap,
    entryFaceId: u32,
    brickmapOrigin: vec3f,
) -> bool {
    let fallbackBrickmapSize = i32(config.fallback_brickmap_size);

    let voxelSize = f32(config.brickmap_size) / f32(fallbackBrickmapSize);
    let origin = localRayOrigin / voxelSize;
    var mapPos = vec3i(floor(origin));

    let deltaDist = voxelSize / abs(rayDirection);
    let rayStep = vec3i(sign(rayDirection));

    var sideDist = (
        vec3f(rayStep) * (vec3f(mapPos) - origin)
        + 0.5 * vec3f(rayStep)
        + 0.5
    ) * deltaDist;
    var faceId = entryFaceId;

    for (var i = 0; i < fallbackBrickmapSize * 3; i++) {
        if any(mapPos < vec3i(0)) || any(mapPos >= vec3i(fallbackBrickmapSize)) {
            break;
        }

        let voxelIndex = getIndex(mapPos, fallbackBrickmapSize);
        if (brickmap.solid_mask & (1u << voxelIndex)) != 0u {
            let packedColors = brickmap.colors[voxelIndex / 4u];
            let paletteIndex = (packedColors >> ((voxelIndex % 4u) * 8u)) & 0xffu;
            let shadowPosition = voxelFaceCenter(brickmapOrigin, mapPos, voxelSize, faceId);
            writeSurfaceHit(position, color_palette[paletteIndex], faceId, shadowPosition);
            return true;
        }

        let mask = sideDist <= min(sideDist.yzx, sideDist.zxy);
        faceId = faceIdFromNormal(-vec3i(mask) * rayStep);
        sideDist += vec3f(mask) * deltaDist;
        mapPos += vec3i(mask) * rayStep;
    }

    return false;
}

fn voxelFaceCenter(brickmapOrigin: vec3f, mapPos: vec3i, voxelSize: f32, faceId: u32) -> vec3f {
    return brickmapOrigin
        + (vec3f(mapPos) + vec3f(0.5)) * voxelSize
        + normalFromFaceId(faceId) * (voxelSize * 0.5);
}

fn writeSurfaceHit(position: vec2u, color: u32, faceId: u32, shadowPosition: vec3f) {
    var visibility = 1u;
    if faceId != 0u {
        if raytraceShadow(shadowPosition, normalFromFaceId(faceId)) {
            visibility = 0u;
        }
    }

    textureStore(base_color_texture, position, vec4f(unpackColor(color), 1.0));
    textureStore(surface_data_texture, position, vec4u(faceId, visibility, 0u, 0u));
}

fn raytraceShadow(hitPosition: vec3f, normal: vec3f) -> bool {
    const directionEpsilon = vec3f(0.000001);

    let rayDirection = -lights.direction.xyz;
    if dot(normal, rayDirection) <= 0.0 {
        return false;
    }

    let brickgridSize = i32(config.brickgrid_size);
    let brickmapSize = f32(config.brickmap_size);
    let gridRange = (brickgridSize - 1) / 2;

    let rayOrigin = hitPosition + normal * 0.001;

    let brickmapRayOrigin = rayOrigin / brickmapSize;
    let brickmapPosition = vec3i(floor(brickmapRayOrigin));
    let cameraBrickmapPosition = vec3i(floor(camera.position.xyz / brickmapSize));
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
            stepPosition.x - cameraBrickmapPosition.x + gridRange,
            cameraBrickmapPosition.y - stepPosition.y + gridRange,
            cameraBrickmapPosition.z - stepPosition.z + gridRange,
        );

        if any(gridPosition < vec3i(0)) || any(gridPosition >= vec3i(brickgridSize)) {
            return false;
        }

        let index = getIndex(gridPosition, brickgridSize);
        let brickgridCell = atomicLoad(&brickgrid[index]);

        if brickgridCell != 0 {
            const unloadedBit = 1u << 24u;
            const requestedBit = 1u << 25u;

            let localRayOrigin = clamp(
                rayOrigin
                    + rayDirection * rayEntryDistance
                    - vec3f(stepPosition) * brickmapSize,
                vec3f(0.0),
                vec3f(brickmapSize - 0.000001),
            );

            if (brickgridCell & unloadedBit) == 0u {
                let brickmap = brickmaps[brickgridCell - 1u];

                if raytraceShadowBrickmap(localRayOrigin, traversalDirection, brickmap) {
                    return true;
                }
            } else {
                let previous = atomicOr(&brickgrid[index], requestedBit);

                if (previous & requestedBit) == 0u {
                    let requestIndex = atomicAdd(&feedback.count, 1u);

                    if requestIndex < 4096u {
                        feedback.data[requestIndex] = index;
                    }
                }

                if raytraceShadowFallbackBrickmap(
                    localRayOrigin,
                    traversalDirection,
                    fallback_brickmaps[index],
                ) {
                    return true;
                }
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

    return false;
}

fn raytraceShadowBrickmap(
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
            return true;
        }

        let mask = sideDist <= min(sideDist.yzx, sideDist.zxy);

        sideDist += vec3f(mask) * deltaDist;
        mapPos += vec3i(mask) * rayStep;
    }

    return false;
}

fn raytraceShadowFallbackBrickmap(
    localRayOrigin: vec3f,
    rayDirection: vec3f,
    brickmap: FallbackBrickmap,
) -> bool {
    let fallbackBrickmapSize = i32(config.fallback_brickmap_size);
    let voxelSize = f32(config.brickmap_size) / f32(fallbackBrickmapSize);
    let origin = localRayOrigin / voxelSize;
    var mapPos = vec3i(floor(origin));
    let rayStep = vec3i(sign(rayDirection));
    let deltaDist = voxelSize / abs(rayDirection);
    var sideDist = (
        vec3f(rayStep) * (vec3f(mapPos) - origin)
        + 0.5 * vec3f(rayStep)
        + 0.5
    ) * deltaDist;

    for (var i = 0; i < fallbackBrickmapSize * 3; i++) {
        if any(mapPos < vec3i(0)) || any(mapPos >= vec3i(fallbackBrickmapSize)) {
            break;
        }

        let voxelIndex = getIndex(mapPos, fallbackBrickmapSize);
        if (brickmap.solid_mask & (1u << voxelIndex)) != 0u {
            return true;
        }

        let mask = sideDist <= min(sideDist.yzx, sideDist.zxy);
        sideDist += vec3f(mask) * deltaDist;
        mapPos += vec3i(mask) * rayStep;
    }

    return false;
}

fn getIndex(position: vec3i, size: i32) -> u32 {
    return u32(position.x + position.y * size + position.z * size * size);
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

fn faceIdFromNormal(normal: vec3i) -> u32 {
    if normal.x < 0 { return 1u; }
    if normal.x > 0 { return 2u; }
    if normal.y < 0 { return 3u; }
    if normal.y > 0 { return 4u; }
    if normal.z < 0 { return 5u; }
    if normal.z > 0 { return 6u; }
    return 0u;
}

fn normalFromFaceId(faceId: u32) -> vec3f {
    switch faceId {
        case 1u: { return vec3f(-1.0, 0.0, 0.0); }
        case 2u: { return vec3f(1.0, 0.0, 0.0); }
        case 3u: { return vec3f(0.0, -1.0, 0.0); }
        case 4u: { return vec3f(0.0, 1.0, 0.0); }
        case 5u: { return vec3f(0.0, 0.0, -1.0); }
        case 6u: { return vec3f(0.0, 0.0, 1.0); }
        default: { return vec3f(0.0); }
    }
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
    let color = textureSample(base_color_sampled, base_color_sampler, input.uv);

    let size = textureDimensions(surface_data_sampled);
    let pixel = min(vec2u(input.uv * vec2f(size)), size - vec2u(1u));

    let surface = textureLoad(surface_data_sampled, vec2i(pixel), 0);

    var crt_effect = 1.0;
    if config.crt == 1 {
        let banding = abs(sin(input.position.x * 5.0));
        crt_effect = mix(1.0, banding, 0.3);
    }

    let faceId = surface.r;
    if faceId == 0u {
        return vec4f(color.rgb * crt_effect, color.a);
    }

    let normal = normalFromFaceId(faceId);

    let directionToLight = -lights.direction.xyz;
    let diffuse = max(dot(normal, directionToLight), 0.0);

    let ambient = lights.ambient_light.rgb * lights.ambient_light.a;
    let directional = lights.directional_light.rgb * lights.directional_light.a * diffuse;
    let shadow = f32(surface.g);

    return vec4f(color.rgb * (ambient + directional * shadow) * crt_effect, color.a);
}

