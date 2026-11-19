// A sphere and a floor, ray marched, with loops, functions, structs,
// const arrays, matrices and derivatives: the usual demoscene toolbox.
#define MAX_STEPS 96
#define SURF 0.001

const vec3 PALETTE[3] = vec3[](vec3(1.0, 0.4, 0.2), vec3(0.2, 0.6, 1.0), vec3(0.9));

struct Hit { float dist; int material; };

mat2 rot(float a) { float c = cos(a), s = sin(a); return mat2(c, -s, s, c); }

Hit scene(vec3 p)
{
    vec3 q = p - vec3(0.0, 1.0, 0.0);
    q.xz *= rot(iTime);
    float sphere = length(q) - 1.0 + 0.05 * sin(10.0 * q.x);
    float plane = p.y;
    return sphere < plane ? Hit(sphere, 0) : Hit(plane, 1);
}

vec3 normalAt(vec3 p)
{
    vec2 e = vec2(SURF, 0.0);
    return normalize(vec3(scene(p + e.xyy).dist - scene(p - e.xyy).dist,
                          scene(p + e.yxy).dist - scene(p - e.yxy).dist,
                          scene(p + e.yyx).dist - scene(p - e.yyx).dist));
}

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    vec2 uv = (2.0 * fragCoord - iResolution.xy) / iResolution.y;
    vec3 ro = vec3(0.0, 1.5, -4.0);
    vec3 rd = normalize(vec3(uv, 1.5));
    float t = 0.0;
    Hit hit = Hit(1e9, 2);
    for (int i = 0; i < MAX_STEPS; i++) {
        hit = scene(ro + rd * t);
        if (hit.dist < SURF || t > 50.0) break;
        t += hit.dist;
    }
    vec3 col = PALETTE[2] * (0.6 + 0.4 * rd.y);
    if (t < 50.0) {
        vec3 p = ro + rd * t;
        float light = max(dot(normalAt(p), normalize(vec3(1.0, 2.0, -1.0))), 0.0);
        col = PALETTE[hit.material] * (0.2 + 0.8 * light);
        col *= 1.0 - 0.5 * smoothstep(0.0, 0.05, fwidth(t) * 0.01);
    }
    fragColor = vec4(pow(col, vec3(0.4545)), 1.0);
}
