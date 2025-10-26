use nalgebra_glm as glm;

pub type Vec3 = glm::Vec3;
pub type Mat4 = glm::Mat4;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, up: Vec3, fov: f32, aspect: f32) -> Self {
        Camera {
            position,
            target,
            up,
            fov,
            aspect,
        }
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        let view = glm::look_at(&self.position, &self.target, &self.up);
        let proj = glm::perspective(self.aspect, self.fov, 0.1, 100.0);
        let inv_view = glm::inverse(&view);
        let inv_proj = glm::inverse(&proj);

        let near_point = glm::vec4_to_vec3(&(inv_proj * glm::vec4(u * 2.0 - 1.0, v * 2.0 - 1.0, -1.0, 1.0)));
        let world_near = glm::vec4_to_vec3(&(inv_view * glm::vec4(near_point.x, near_point.y, near_point.z, 1.0)));
        let dir = glm::normalize(&(world_near - self.position));

        Ray {
            origin: self.position,
            direction: dir,
        }
    }
}

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Sphere { center, radius }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<f32> {
        let oc = ray.origin - self.center;
        let a = glm::dot(&ray.direction, &ray.direction);
        let b = 2.0 * glm::dot(&oc, &ray.direction);
        let c = glm::dot(&oc, &oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 {
                Some(t)
            } else {
                None
            }
        }
    }

    pub fn normal_at(&self, point: &Vec3) -> Vec3 {
        glm::normalize(&(point - self.center))
    }
}