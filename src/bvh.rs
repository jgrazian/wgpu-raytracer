use crate::aabb::{Bounded, AABB};
use crate::geometry::Sphere;
use crate::traits::AsBytes;
use glam::Vec3;

// https://www.ks.uiuc.edu/Research/vmd/projects/ece498/raytracing/GPU_BVHthesis.pdf
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum Leaf {
    S(Sphere),
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Node {
    bb_min: Vec3,
    node_type: u32,
    bb_max: Vec3,
    esc_index: u32,
}
unsafe impl bytemuck::Pod for Node {}
unsafe impl bytemuck::Zeroable for Node {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum BVHElement {
    Node(Node),
    Leaf(Leaf),
}
unsafe impl bytemuck::Pod for BVHElement {}
unsafe impl bytemuck::Zeroable for BVHElement {}

#[derive(Debug)]
pub struct BVH {
    pub nodes: Vec<BVHElement>,
}

impl BVH {
    pub fn from_spheres(objects: &[Sphere]) -> Self {
        let mut leaves = Vec::with_capacity(objects.len());
        for obj in objects {
            leaves.push(Leaf::S(*obj));
        }

        Self::new(leaves.as_slice())
    }

    pub fn new(objects: &[Leaf]) -> Self {
        let mut index: Vec<usize> = (0..objects.len()).collect();
        let mut nodes: Vec<BVHElement> = Vec::with_capacity(objects.len() * 2);

        Self::build_node(&mut nodes, objects, &mut index, 1);

        let nodes_len = nodes.len() as u32;
        for node in nodes.iter_mut() {
            if node.get_esc_index() >= nodes_len {
                node.set_esc_index(0xFFFFFFFF);
            }
        }

        BVH { nodes }
    }

    fn build_node(
        nodes: &mut Vec<BVHElement>,
        objects: &[Leaf],
        index: &mut Vec<usize>,
        esc_index: u32,
    ) -> usize {
        if index.len() == 1 {
            nodes.push(objects[index[0]].as_element(esc_index));
            return 1;
        }

        let mut bounds = objects[index[0]].get_bounds();
        for i in index.iter() {
            bounds.extend(&objects[*i].get_bounds());
        }

        let (mut li, mut ri) = Self::split(objects, index, &bounds);

        nodes.push(BVHElement::Node(Node {
            bb_min: bounds.min,
            node_type: 0xFFFFFFFF,
            bb_max: bounds.max,
            esc_index: 0,
        }));

        let start_index = nodes.len() - 1;

        let num_l = Self::build_node(nodes, objects, &mut li, esc_index + 1);
        let num_r = Self::build_node(nodes, objects, &mut ri, esc_index + num_l as u32 + 1);

        let e = (start_index + num_l + num_r + 1) as u32;
        nodes[start_index].set_esc_index(e);
        if num_r == 1 {
            nodes[start_index + num_l + 1].set_esc_index(e);
        }

        num_l + num_r + 1
    }

    fn split<'a>(
        objects: &'a [Leaf],
        index: &'a mut Vec<usize>,
        bounds: &'a AABB,
    ) -> (Vec<usize>, Vec<usize>) {
        let bounds_axis = bounds.largest_axis();

        let sort = |a: &usize, b: &usize| {
            let axis0 = axis_size(objects[*a].get_bounds().center(), bounds_axis);
            let axis1 = axis_size(objects[*b].get_bounds().center(), bounds_axis);

            axis0.partial_cmp(&axis1).unwrap()
        };

        index.sort_by(sort);

        let (l, r) = index.split_at(index.len() / 2);
        (l.to_vec(), r.to_vec())
    }
}

fn axis_size(v: Vec3, axis: Vec3) -> f32 {
    let d = v * axis;
    match (d.x() != 0.0, d.y() != 0.0, d.z() != 0.0) {
        (true, false, false) => return d.x(),
        (false, true, false) => return d.y(),
        (false, false, true) => return d.z(),
        _ => return 0.0,
    }
}

impl Bounded for Leaf {
    fn get_bounds(&self) -> AABB {
        match self {
            Leaf::S(s) => s.get_bounds(),
        }
    }
}

impl Bounded for BVHElement {
    fn get_bounds(&self) -> AABB {
        return match self {
            BVHElement::Node(n) => AABB {
                min: n.bb_min,
                max: n.bb_max,
            },
            BVHElement::Leaf(l) => l.get_bounds(),
        };
    }
}

impl Leaf {
    fn as_element(&self, esc_index: u32) -> BVHElement {
        return match self {
            Leaf::S(s) => BVHElement::Leaf(Leaf::S(Sphere { esc_index, ..*s })),
        };
    }
}

impl BVHElement {
    fn set_esc_index(&mut self, esc_index: u32) {
        match self {
            BVHElement::Node(n) => n.esc_index = esc_index,
            BVHElement::Leaf(l) => match l {
                Leaf::S(s) => s.esc_index = esc_index,
            },
        }
    }

    fn get_esc_index(&self) -> u32 {
        return match self {
            BVHElement::Node(n) => n.esc_index,
            BVHElement::Leaf(l) => match l {
                Leaf::S(s) => s.esc_index,
            },
        };
    }
}

impl AsBytes for BVH {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for node in &self.nodes {
            match node {
                BVHElement::Node(n) => bytes.extend_from_slice(bytemuck::bytes_of(n)),
                BVHElement::Leaf(l) => match l {
                    Leaf::S(s) => bytes.extend_from_slice(bytemuck::bytes_of(s)),
                },
            };
        }

        bytes
    }

    fn bytes_size(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct State {
        a: u32,
    }
    fn xorshift32(state: &mut State) -> u32 {
        let mut x = state.a;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        state.a = x;
        x
    }
    fn randf(state: &mut State) -> f32 {
        (xorshift32(state) as f32) / (u32::MAX as f32)
    }
    fn rng(state: &mut State, r: f32) -> f32 {
        randf(state) * r
    }

    #[test]
    fn test() {
        let mut s = State { a: 35924 };
        let mut objects = Vec::with_capacity(1_000_000);
        for _ in 0..1_000_000 {
            objects.push(Leaf::S(Sphere {
                center: Vec3::new(rng(&mut s, 100.0), rng(&mut s, 100.0), rng(&mut s, 100.0)),
                radius: 1.0,
                mat_index: 1,
                pad0: [0.0, 0.0],
                esc_index: 0,
            }))
        }
        println!("Made spheres");

        let bvh = BVH::new(objects.as_mut_slice());
        println!("{:?}", bvh);
    }
}
