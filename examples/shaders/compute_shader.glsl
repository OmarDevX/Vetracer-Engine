#version 460 core

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D screen;

const int max_objects = 10;
const int num_motion_blur_samples = 5; // Reduce number of samples for better performance
const int bounces = 5;
struct Ray {
    vec3 Origin;
    vec3 Direction;
};

uniform Ray ray;
uniform vec3 skycolor;
uniform vec3 camera_pos;
uniform vec3 camera_front;
uniform vec3 camera_up;
uniform vec3 camera_right;
uniform vec3 camera_velocity;
uniform float fov; // Field of View in radians
uniform bool is_fisheye;
float focal_length = 5.0; // Focal length for depth of field
float aperture = 0.01; // Aperture size for depth of field

// Objects
uniform vec3 objects_size[max_objects]; // Use this for cube dimensions
uniform vec3 objects_color[max_objects];
uniform vec3 objects_position[max_objects];
uniform float objects_radius[max_objects]; // Use this for sphere radius
uniform float objects_roughness[max_objects];
uniform float objects_emission[max_objects];
uniform bool is_cube[max_objects];
uniform bool is_glass[max_objects];
uniform float objects_refractive_index[max_objects];

const int max_triangles = 20; // Maximum number of triangles in the scene
// Meshes
uniform vec3 vertices[max_triangles * 3];
uniform vec3 normals[max_triangles * 3];
uniform ivec3 triangles[max_triangles];
// Instead of fetching from uniforms, hardcode these values
vec3 albedo = vec3(0.5); // Gray color
float roughness = 0.5;  // Roughness set to 0.5
float emission = 0.0;   // Emission set to 0.0

uniform bool is_accumulation;
uniform float currentTime; // Uniform variable to receive current time from application
uniform int frameNumber; // Uniform variable for the current frame number

const float pi = 3.1415926535897932385;

// Random number generation using pcg32i_random_t, using inc = 1. Our random state is a uint.
uint stepRNG(uint rngState)
{
    return rngState * 747796405 + 1;
}

// Steps the RNG and returns a floating-point value between 0 and 1 inclusive.
float stepAndOutputRNGFloat(inout uint rngState)
{
    // Condensed version of pcg_output_rxs_m_xs_32_32, with simple conversion to floating-point [0,1].
    rngState  = stepRNG(rngState);
    uint word = ((rngState >> ((rngState >> 28) + 4)) ^ rngState) * 277803737;
    word      = (word >> 22) ^ word;
    return float(word) / 4294967295.0f;
}

float random(inout uint rngState)
{
    return stepAndOutputRNGFloat(rngState);
}

float random(inout uint rngState, float min, float max)
{
    // Returns a random real in [min,max).
    return min + (max - min) * random(rngState);
}

vec3 random_in_unit_sphere(inout uint rngState)
{
    vec3 p = vec3(random(rngState, -1.0, 1.0), random(rngState, -1.0, 1.0), random(rngState, -1.0, 1.0));
    return normalize(p);
}

vec3 random_in_hemisphere(inout uint rngState, vec3 normal)
{
    vec3 in_unit_sphere = random_in_unit_sphere(rngState);
    if (dot(in_unit_sphere, normal) > 0.0) // In the same hemisphere as the normal
        return in_unit_sphere;
    else
        return -in_unit_sphere;
}

vec3 random_cosine_direction(inout uint rngState)
{
    float r1 = random(rngState);
    float r2 = random(rngState);
    float z = sqrt(1.0 - r2);

    float phi = 2.0 * pi * r1;
    float x = cos(phi) * sqrt(r2);
    float y = sin(phi) * sqrt(r2);

    return vec3(x, y, z);
}

float schlick(float cosine, float ref_idx)
{
    float r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * pow((1.0 - cosine), 5.0);
}

bool refract(vec3 v, vec3 n, float ni_over_nt, inout vec3 refracted)
{
    vec3 uv = normalize(v);
    float dt = dot(uv, n);
    float discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);
    if (discriminant > 0.0)
    {
        refracted = ni_over_nt * (uv - n * dt) - n * sqrt(discriminant);
        return true;
    }
    else
        return false;
}

vec3 reflect(vec3 v, vec3 n)
{
    return v - 2.0 * dot(v, n) * n;
}

void applyBloom(inout vec3 color, vec3 light, float threshold, float intensity)
{
    vec3 bloomColor = max(vec3(0.0), light - threshold);
    bloomColor *= intensity;
    color += bloomColor;
}
vec3 calculateLightContribution(vec3 rayOrigin, vec3 rayDir, inout uint rngState, vec3 contribution)
{
    vec3 light = vec3(0.0);

    for (int bounce = 0; bounce < bounces; ++bounce) // Reduce number of bounces for better performance
    {
        // Track closest object information
        float closestIntersection = 9999.0;
        int closestObjectIndex = -1;
        bool isSphere = true; // To distinguish between spheres and cubes
        float objectRoughness = 0.0;

        // Find closest object intersection
        for (int i = 0; i < max_objects; ++i)
        {
            if (is_cube[i])
            {
                // Find closest cube intersection
                vec3 cube_position = objects_position[i];
                vec3 cube_size = objects_size[i];
                vec3 cube_min = cube_position - cube_size * 0.5;
                vec3 cube_max = cube_position + cube_size * 0.5;

                float tMin = (cube_min.x - rayOrigin.x) / rayDir.x;
                float tMax = (cube_max.x - rayOrigin.x) / rayDir.x;
                if (tMin > tMax) { float temp = tMin; tMin = tMax; tMax = temp; }

                float tyMin = (cube_min.y - rayOrigin.y) / rayDir.y;
                float tyMax = (cube_max.y - rayOrigin.y) / rayDir.y;
                if (tyMin > tyMax) { float temp = tyMin; tyMin = tyMax; tyMax = temp; }

                if ((tMin > tyMax) || (tyMin > tMax))
                    continue;

                if (tyMin > tMin)
                    tMin = tyMin;
                if (tyMax < tMax)
                    tMax = tyMax;

                float tzMin = (cube_min.z - rayOrigin.z) / rayDir.z;
                float tzMax = (cube_max.z - rayOrigin.z) / rayDir.z;
                if (tzMin > tzMax) { float temp = tzMin; tzMin = tzMax; tzMax = temp; }

                if ((tMin > tzMax) || (tzMin > tMax))
                    continue;

                if (tzMin > tMin)
                    tMin = tzMin;
                if (tzMax < tMax)
                    tMax = tzMax;

                if (tMin < 0) tMin = tMax;

                if (tMin > 0 && tMin < closestIntersection)
                {
                    vec3 hit_point = rayOrigin + rayDir * tMin;

                    // Calculate the normal of the intersected face
                    vec3 normal;

                    if (abs(hit_point.x - cube_min.x) < 0.001) normal = vec3(-1, 0, 0);
                    else if (abs(hit_point.x - cube_max.x) < 0.001) normal = vec3(1, 0, 0);
                    else if (abs(hit_point.y - cube_min.y) < 0.001) normal = vec3(0, -1, 0);
                    else if (abs(hit_point.y - cube_max.y) < 0.001) normal = vec3(0, 1, 0);
                    else if (abs(hit_point.z - cube_min.z) < 0.001) normal = vec3(0, 0, -1);
                    else if (abs(hit_point.z - cube_max.z) < 0.001) normal = vec3(0, 0, 1);

                    // Use this hit information for shading
                    closestIntersection = tMin;
                    closestObjectIndex = i;
                    objectRoughness = objects_roughness[i];
                    isSphere = false; // Mark it as cube intersection
                }
            }
            else
            {
                // Find closest sphere intersection
                vec3 sphere_position = objects_position[i];
                float sphere_radius = objects_radius[i];

                vec3 oc = rayOrigin - sphere_position;
                float a = dot(rayDir, rayDir);
                float b = 2.0 * dot(oc, rayDir);
                float c = dot(oc, oc) - sphere_radius * sphere_radius;
                float discriminant = b * b - 4.0 * a * c;

                if (discriminant > 0.0)
                {
                    float temp = (-b - sqrt(discriminant)) / (2.0 * a);
                    if (temp > 0.0 && temp < closestIntersection)
                    {
                        closestIntersection = temp;
                        closestObjectIndex = i;
                        objectRoughness = objects_roughness[i];
                        isSphere = true;
                    }
                }
            }
        }

        // Handle intersection and shading
        if (closestObjectIndex != -1)
        {
            vec3 hit_point;
            vec3 normal;
            vec3 albedo;
            float emission;

            if (isSphere)
            {
                vec3 sphere_position = objects_position[closestObjectIndex];
                float sphere_radius = objects_radius[closestObjectIndex];
                hit_point = rayOrigin + rayDir * closestIntersection;
                normal = normalize(hit_point - sphere_position);
                albedo = objects_color[closestObjectIndex] / 255.0;
                emission = objects_emission[closestObjectIndex];
            }
            else
            {
                vec3 cube_position = objects_position[closestObjectIndex];
                vec3 cube_size = objects_size[closestObjectIndex];
                vec3 cube_min = cube_position - cube_size * 0.5;
                vec3 cube_max = cube_position + cube_size * 0.5;
                hit_point = rayOrigin + rayDir * closestIntersection;

                // Calculate normal for cube
                if (abs(hit_point.x - cube_min.x) < 0.001) normal = vec3(-1, 0, 0);
                else if (abs(hit_point.x - cube_max.x) < 0.001) normal = vec3(1, 0, 0);
                else if (abs(hit_point.y - cube_min.y) < 0.001) normal = vec3(0, -1, 0);
                else if (abs(hit_point.y - cube_max.y) < 0.001) normal = vec3(0, 1, 0);
                else if (abs(hit_point.z - cube_min.z) < 0.001) normal = vec3(0, 0, -1);
                else if (abs(hit_point.z - cube_max.z) < 0.001) normal = vec3(0, 0, 1);

                albedo = objects_color[closestObjectIndex] / 255.0;
                emission = objects_emission[closestObjectIndex];
            }

            // Calculate reflection direction based on roughness
            vec3 reflected = reflect(rayDir, normal);
            float reflectivity = mix(1.0, 0.0, objectRoughness); // Convert roughness to reflectivity

            vec3 random_direction = random_in_unit_sphere(rngState);

            // Adjust randomness based on roughness
            vec3 adjusted_reflection = mix(reflected, random_direction, objectRoughness);

            // Determine if the object is reflective enough to reflect
            if (random(rngState) < reflectivity)
            {
                rayDir = adjusted_reflection;
                rayOrigin = hit_point + rayDir * 0.001;
            }
            else
            {
                // Non-reflective path (e.g., Lambertian shading)
                rayOrigin = hit_point + normal * 0.001;
                rayDir = normalize(normal + random_in_unit_sphere(rngState));

                // Update contribution with current object's albedo
                contribution *= albedo;

                // Accumulate light based on object albedo and emission
                light += albedo * emission * contribution;
            }
        }
        else
        {
            // No object intersection (use sky color)
            vec3 skyColor = skycolor;
            light += skyColor * contribution;
            break; // Exit the loop since no further reflections should be considered
        }
        
    }

    return light;
}

void main()
{
    ivec2 texel_coords = ivec2(gl_GlobalInvocationID.xy);
    vec2 screen_resolution = vec2(imageSize(screen));
    vec2 normalized_coords = (vec2(texel_coords) + vec2(0.5)) / screen_resolution * 2.0 - 1.0;
    float aspect_ratio = screen_resolution.x / screen_resolution.y;

    vec3 initial_rayDir;
    vec3 initial_rayOrigin;

    if (is_fisheye) {
        // Fisheye camera effect using a polynomial function
        float r = length(normalized_coords);
        float theta = atan(r);
        float fisheye_factor = 1.0 + (r * r) * 0.2; // adjust this value to control the fisheye effect
        vec2 distorted_coords = normalized_coords * fisheye_factor;

        // Compute ray direction using camera vectors with FOV adjustment
        float scale = tan(fov * 0.5);
        initial_rayDir = normalize(camera_front + distorted_coords.x * aspect_ratio * scale * camera_right + distorted_coords.y * scale * camera_up);

        // Rotate ray direction to match camera rotation
        initial_rayDir = normalize(dot(initial_rayDir, camera_right) * camera_right + dot(initial_rayDir, camera_up) * camera_up + dot(initial_rayDir, camera_front) * camera_front);

        initial_rayOrigin = camera_pos;
    } else {
        // Normal rendering without fisheye effect
        float scale = tan(fov * 0.5);
        initial_rayDir = normalize(camera_front + normalized_coords.x * aspect_ratio * scale * camera_right + normalized_coords.y * scale * camera_up);
        initial_rayOrigin = camera_pos;
    }

    vec3 accumulated_light = vec3(0.0);
    uint rngState = (uint(gl_GlobalInvocationID.x) * 1973u + uint(gl_GlobalInvocationID.y) * 9277u + uint(frameNumber) * 26699u + uint(currentTime * 1000.0));

    // Depth of Field (DoF) calculations
    vec3 focal_point = initial_rayOrigin + initial_rayDir * focal_length;

    for (int i = 0; i < num_motion_blur_samples; ++i)
    {
        // Motion blur: Jitter ray origin and direction based on camera velocity
        float t = random(rngState);
        vec3 rayOrigin = initial_rayOrigin + (t * camera_velocity) / 2;
        vec3 rayDir = normalize(focal_point - rayOrigin);
        vec3 aperture_offset = aperture * random_in_unit_sphere(rngState);
        rayOrigin += aperture_offset;
        rayDir = normalize(focal_point - rayOrigin);

        // Calculate light contribution (including glass handling)
        vec3 light = calculateLightContribution(rayOrigin, rayDir, rngState, vec3(1.0));
        accumulated_light += light;
    }

    // Average the light contributions from all samples
    vec3 final_light = accumulated_light / float(num_motion_blur_samples);

    // Apply bloom
    float bloomThreshold = 0.8;
    float bloomIntensity = 1.0;
    applyBloom(final_light, final_light, bloomThreshold, bloomIntensity);

    if (is_accumulation)
    {
        vec4 prevColor = imageLoad(screen, texel_coords);
        float numFrames = prevColor.a;
        numFrames += 1.0;
        vec3 accumulatedColor = (prevColor.rgb * prevColor.a + final_light) / numFrames;
        // Output final color to screen texture with accumulation
        imageStore(screen, texel_coords, vec4(accumulatedColor, numFrames));
    }
    else
    {
        // Directly output the current frame color
        imageStore(screen, texel_coords, vec4(final_light, 1.0));
    }
}
