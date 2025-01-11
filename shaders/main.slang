static const float INF = 3.40282346638528859812e+38;
static const float NINF = -3.40282346638528859812e+38;
static const float PI      = 3.141592653589793238462;
static const float EPSILON = 1e-2;
static const uint MAX_DEPTH = 10u;

static const float2 vertices[6] = {
  float2(-1, 1),
  float2(-1, -1),
  float2(1, 1),
  float2(1, 1),
  float2(-1, -1),
  float2(1, -1)
};

func near_zero(float3 v) -> bool{
  let t = 1e-8;
  let a = abs(v);
  return a.x < t && a.y < t && a.z < t;
}

struct Uniforms {
  uint width;
  uint height;
  uint frame_count;
  CameraUniforms camera;
};

struct CameraUniforms{
  float3 origin;
  float3 u;
  float3 v;
  float3 w;
}

struct HitRecord {
  float3 p;
  float3 n;
  float t;
  float3 color;
  bool is_front_face;

  [mutating]
  func set_front_face(Ray r, float3 n){
    is_front_face = dot(r.dir, n) < 0.0;
    n = is_front_face ? n : -n;
  }

};

struct Ray {
  float3 origin;
  float3 dir;
  func at(float t) -> float3 {
    return origin + dir * t;
  }
};

struct Sphere {
  float3 center;
  float radius;
  float3 color;

  func hit(Ray ray, Interval ray_t) -> Optional<HitRecord> {
    let v = center - ray.origin;
    let a = dot(ray.dir, ray.dir);
    let b = dot(v, ray.dir);
    let c = dot(v, v) - radius * radius;
    let d = b * b - a * c;

    if (d < 0.0f) {
      return none;
    }

    let rtd = sqrt(d);
    let inva = 1.0f / a;

    var root = (b-rtd) * inva;
    if (!ray_t.surrounds(root)){
      root = (b + rtd) / a;
      if (!ray_t.surrounds(root)){
        return none;
      }
    }
    float3 p = ray.at(root);
    float3 N = (p - center) / radius;

    HitRecord record;
    record.p = p;
    record.t = root;
    record.color = color;
    record.set_front_face(ray, N);
    return record;
  }
}

struct Interval{
  float min;
  float max;

  bool contains(float x) {
    return min <= x && x <= max;
  }

  bool surrounds(float x) {
    return min < x && x < max;
  }
  static const This EMPTY = {INF, NINF};
  static const This ALL = {NINF, INF};
  static const This IN_SCENE = {EPSILON, INF};
}

struct ScatterRecord{
  float3 attentuation;
  Ray ray;
}

static Rng rng;

struct Rng{
  uint state;

  static func init(uint2 pixel) -> Rng{
    let pixel = int2(pixel);
    let seed = (pixel.x + pixel.y * uniforms.width) ^ jenkins_hash(uniforms.frame_count);
    return { jenkins_hash(seed) };
  }

  [mutating]
  func xorshift() -> uint{
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    return state;
  }

  [mutating]
  func rand() -> float{
    return reinterpret<float>(0x3f800000u | (xorshift() >> 9u)) - 1.0;
  }

  [mutating]
  func rand_vec3() -> float3{
    return float3(rand(), rand(), rand());
  }

  [mutating]
  func rand_nvec3() -> float3{
    let phi = 2*PI*rand();
    let z = 1 - 2 * rand();
    let sinz = sqrt(1-z*z);
    let x=cos(phi)*sinz;
    let y=sin(phi)*sinz;
    return float3(x, y, z);
  }

  [mutating]
  func rand_along_vector(float3 n) -> float3{
    let v = rand_nvec3();
    return select(dot(n, v) > 0, v, -v);
  }

  static func jenkins_hash(uint i) -> uint {
    var x = i;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
  }
}

[[vk::binding(0, 0)]]
ConstantBuffer<Uniforms> uniforms;

[[vk::binding(1, 0)]]
[[format("rgba32f")]]
RWTexture2D<float4> randiance_samples;

static const Sphere initialScene[] = {
  { float3(0, 0, -1), 0.5f, float3(0.7, 0.2, 0.1) },
  { float3(1.4, 0.5, -1), 1.0f, float3(0.9, 0.9, 1.0) },
  { float3(0, -100.5f, -1), 100.0f, float3(0.8, 0.8, 0.0) }
};


func hit_scene(Ray ray, Interval ray_t) -> Optional<HitRecord>{
  Optional<HitRecord> closest_hit = none;
  var least_t = ray_t.max;

  Sphere tmp;
  for (uint i = 0; i < initialScene.getCount(); i++) {
    tmp = initialScene[i];
    let hit = tmp.hit(ray, {ray_t.min, least_t});
    if (hit.hasValue){
      closest_hit = hit;
      least_t = closest_hit.value.t;
    }
  }
  return closest_hit;
}

func skyColor(Ray ray) -> float3{
  float t = 0.5f * (normalize(ray.dir).y + 1.0f);
  return lerp(float3(1, 1, 1), float3(0.3f, 0.5f, 1.0f), t);
}

func scatter(Ray ray_in, HitRecord hit, inout Rng rng) -> ScatterRecord{
  let deflect_dir = rng.rand_nvec3() + hit.n;
  let attentuation = hit.color;
  Ray ray = { ray_in.at(hit.t) , deflect_dir};
  return {attentuation, ray};
}


[[shader("vertex")]]
float4 display_vs(uint vid : SV_VertexID) : SV_Position {
  return float4(vertices[vid], 0.0f, 1.0f);
}

[[shader("pixel")]]
float4 display_fs(float4 pos : SV_Position) : SV_Target0 {
  var rng = Rng.init(uint2(pos.xy));
  float aspect_ratio = float(uniforms.width) / float(uniforms.height);
  float3 origin = uniforms.camera.origin;
  float focus_distance = 1.0f;

  let offset = float2(rng.rand() - 0.5, rng.rand() - 0.5);
  float2 uv = (pos.xy + offset)/ float2(float(uniforms.width - 1), float(uniforms.height - 1));
  uv = (2.0f * uv - float2(1.0f, 1.0f)) * float2(aspect_ratio, -1.0f);

  let camera_rot = float3x3(uniforms.camera.u, uniforms.camera.v, uniforms.camera.w);
  float3 to_viewport = float3(uv, focus_distance);
  let dir = mul(camera_rot, to_viewport);
  Ray ray = {origin, dir};

  uint depth = 0u;
  float3 throughput = float3(1, 1, 1);
  float3 radiance_sample = float3(0, 0, 0);
  while (depth < 20){
    let hit = hit_scene(ray, Interval.IN_SCENE);
    if(!hit.hasValue){
      radiance_sample += throughput*skyColor(ray);
      break;
    }
    let scattered = scatter(ray, hit.value, rng);
    throughput*=scattered.attentuation;
    ray = scattered.ray;
    depth+=1;
  }
  let total = radiance_sample.xyz + select(uniforms.frame_count>1, randiance_samples.Load(int2(pos.xy)).xyz, 0);
  randiance_samples[int2(pos.xy)] = float4(total, 0.0);
  let color = total/float(uniforms.frame_count);
  let final = pow(color, 1/2.2);
  return float4(final, 1.0);
}
