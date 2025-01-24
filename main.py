import random

def random_double(min=0.0, max=1.0):
    return random.uniform(min, max)

def random_vector(min=0.0, max=1.0):
    return [random_double(min, max) for _ in range(3)]

print("static const Sphere initialScene[] = {")

print("  { float3(0, -1000, 0), 1000, Lambertian(float3(0.5, 0.5, 0.5)) },")

W = 1
for a in range(-W, W):
    for b in range(-W, W):
        choose_mat = random_double()
        center = [a + 0.9 * random_double(), 0.2, b + 0.9 * random_double()]

        if (sum([(center[i] - [4, 0.2, 0][i]) ** 2 for i in range(3)]) ** 0.5) > 0.9:
            if choose_mat < 0.8:
                albedo = [x * y for x, y in zip(random_vector(), random_vector())]
                print(f"  {{ float3({center[0]}, {center[1]}, {center[2]}), 0.2, Lambertian(float3({albedo[0]}, {albedo[1]}, {albedo[2]})) }},")
            elif choose_mat < 0.95:
                albedo = random_vector(0.5, 1)
                print(f"  {{ float3({center[0]}, {center[1]}, {center[2]}), 0.2, Metal(float3({albedo[0]}, {albedo[1]}, {albedo[2]})) }},")
            else:
                print(f"  {{ float3({center[0]}, {center[1]}, {center[2]}), 0.2, Dielectric(1.5) }},")

print("  { float3(0, 1, 0), 1.0, Dielectric(1.5) },")
print("  { float3(-4, 1, 0), 1.0, Lambertian(float3(0.4, 0.2, 0.1)) },")
print("  { float3(4, 1, 0), 1.0, Metal(float3(0.7, 0.6, 0.5)) },")

print("};")
