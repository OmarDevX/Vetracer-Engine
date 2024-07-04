#version 460 core

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D screen;

const int max_spheres = 10;
const int num_motion_blur_samples = 8; // Number of samples for motion blur

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
float focal_length = 5.0; // Focal length for depth of field
float aperture = 0.01; // Aperture size for depth of field

uniform vec3 spheres_color[max_spheres];
uniform vec3 spheres_position[max_spheres];
uniform float spheres_radius[max_spheres];
uniform float spheres_roughness[max_spheres];
uniform float spheres_emission[max_spheres];

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

    for (int bounce = 0; bounce < 10; ++bounce)
    {
        // Track closest sphere information
        float closestIntersection = 9999.0;
        int closestSphereIndex = -1;
        float sphereRoughness = 0.0;

        // Find closest sphere intersection
        for (int i = 0; i < max_spheres; ++i)
        {
            vec3 sphere_position = spheres_position[i];
            float sphere_radius = spheres_radius[i];

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
                    closestSphereIndex = i;
                    sphereRoughness = spheres_roughness[i];
                }
            }
        }

        // Handle sphere intersection and shading
        if (closestSphereIndex != -1)
        {
            vec3 sphere_position = spheres_position[closestSphereIndex];
            float sphere_radius = spheres_radius[closestSphereIndex];
            vec3 hit_point = rayOrigin + rayDir * closestIntersection;
            vec3 normal = normalize(hit_point - sphere_position);

            // Calculate reflection direction based on roughness
            vec3 reflected = reflect(rayDir, normal);
            float reflectivity = mix(1.0, 0.0, sphereRoughness); // Convert roughness to reflectivity

            // Determine if the sphere is reflective enough to reflect
            if (random(rngState) < reflectivity)
            {
                rayDir = reflected;
                rayOrigin = hit_point + rayDir * 0.001;
            }
            else
            {
                // Non-reflective path (e.g., Lambertian shading)
                vec3 albedo = spheres_color[closestSphereIndex] / 255.0;
                rayOrigin = hit_point + normal * 0.001;
                rayDir = normalize(normal + random_in_unit_sphere(rngState));

                // Update contribution with current sphere's albedo
                contribution *= albedo;

                // Accumulate light based on sphere albedo and emission
                light += albedo * spheres_emission[closestSphereIndex] * contribution;
            }
        }
        else
        {
            // No sphere intersection (use sky color)
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

    // Compute ray direction using camera vectors with FOV adjustment
    float scale = tan(fov * 0.5);
    vec3 initial_rayDir = normalize(camera_front + normalized_coords.x * aspect_ratio * scale * camera_right + normalized_coords.y * scale * camera_up);
    vec3 initial_rayOrigin = camera_pos;

    vec3 accumulated_light = vec3(0.0);
    uint rngState = (uint(gl_GlobalInvocationID.x) * 1973u + uint(gl_GlobalInvocationID.y) * 9277u + uint(frameNumber) * 26699u + uint(currentTime * 1000.0));

    // Depth of Field (DoF) calculations
    vec3 focal_point = initial_rayOrigin + initial_rayDir * focal_length;

    for (int i = 0; i < num_motion_blur_samples; ++i)
    {
        // Motion blur: Jitter ray origin and direction based on camera velocity
        float t = random(rngState);
        vec3 rayOrigin = initial_rayOrigin + (t * camera_velocity)/2;
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
