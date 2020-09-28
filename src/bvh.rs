use crate::aabb::{Axis, Bounded, AABB};
use crate::geometry::Buffer;
use std::marker::PhantomData;

// https://www.ks.uiuc.edu/Research/vmd/projects/ece498/raytracing/GPU_BVHthesis.pdf

#[derive(Debug, Clone, Copy)]
enum NodeType {
    Node = 0,
    Leaf = 1,
}

#[derive(Debug)]
struct Node {
    bounds: AABB,
    node_type: NodeType,
    ptr: u32,
}

#[derive(Debug)]
pub struct BVH<T: Bounded> {
    nodes: Vec<Node>,
    phantom: PhantomData<T>,
}

impl<T: Bounded + std::fmt::Debug> BVH<T> {
    pub fn from_objects(objects: &Vec<T>) -> Self {
        // Create outermost bounding-box
        let mut bounds = objects[0].get_bounds();
        for obj in objects {
            bounds = AABB::union(&bounds, &obj.get_bounds());
        }

        // 0th node
        let mut nodes = vec![Node::new(bounds, NodeType::Node, 0)];

        // Split into left/right
        let index = (0..objects.len() as u32).collect();
        let objects = objects.iter().collect();
        let (l_obj, l_ind, r_obj, r_ind) =
            Self::split_on_axis(&bounds.largest_axis(), &bounds, &objects, &index);

        // Recurse
        let mut left_nodes = Self::make_bvh(&l_obj, &l_ind, 1);
        nodes.append(&mut left_nodes);
        let mut right_nodes = Self::make_bvh(&r_obj, &r_ind, nodes.len() as u32);
        nodes.append(&mut right_nodes);

        nodes[0] = Node::new(bounds, NodeType::Node, nodes.len() as u32);

        BVH {
            nodes,
            phantom: PhantomData,
        }
    }

    fn make_bvh(objects: &Vec<&T>, index: &Vec<u32>, ptr: u32) -> Vec<Node> {
        let mut bounds = objects[0].get_bounds();

        // Only 1 object left it is a leaf
        if objects.len() == 1 {
            return vec![Node::new(bounds, NodeType::Leaf, index[0])];
        }

        let mut nodes = vec![Node::new(bounds, NodeType::Node, ptr)];

        for obj in objects {
            bounds = AABB::union(&bounds, &obj.get_bounds());
        }

        let mut split_axis = bounds.largest_axis();
        let (mut l_obj, mut l_ind, mut r_obj, mut r_ind) =
            Self::split_on_axis(&split_axis, &bounds, objects, index);

        match (l_obj.len() > 0, r_obj.len() > 0) {
            (false, true) => {
                l_obj = r_obj[0..r_obj.len() / 2].to_vec();
                l_ind = r_ind[0..r_ind.len() / 2].to_vec();
                r_obj = r_obj[r_obj.len() / 2..].to_vec();
                r_ind = r_ind[r_ind.len() / 2..].to_vec();
            }
            (true, false) => {
                l_obj = l_obj[0..l_obj.len() / 2].to_vec();
                l_ind = l_ind[0..l_ind.len() / 2].to_vec();
                r_obj = l_obj[l_obj.len() / 2..].to_vec();
                r_ind = l_ind[l_ind.len() / 2..].to_vec();
            }
            _ => (),
        }

        if l_obj.len() < 3 {
            //println!("{:?}", l_obj);
        }

        let mut left_nodes = Self::make_bvh(&l_obj, &l_ind, ptr + 1);
        nodes.append(&mut left_nodes);
        let mut right_nodes = Self::make_bvh(&r_obj, &r_ind, ptr + nodes.len() as u32);
        nodes.append(&mut right_nodes);

        nodes[0] = Node::new(bounds, NodeType::Node, ptr + nodes.len() as u32);

        nodes
    }

    fn split_on_axis<'a>(
        axis: &Axis,
        bounds: &AABB,
        objects: &'a Vec<&T>,
        index: &Vec<u32>,
    ) -> (Vec<&'a T>, Vec<u32>, Vec<&'a T>, Vec<u32>) {
        let mut left_objects: Vec<&T> = Vec::new();
        let mut left_index: Vec<u32> = Vec::new();
        let mut right_objects: Vec<&T> = Vec::new();
        let mut right_index: Vec<u32> = Vec::new();

        match axis {
            Axis::X => {
                let mid = 0.5 * (bounds.min_x() + bounds.max_x());
                for (i, obj) in (&objects).iter().enumerate() {
                    let above = obj.get_bounds().max_x() - mid;
                    let below = mid - obj.get_bounds().min_x();
                    if above >= below {
                        left_objects.push(obj);
                        left_index.push(index[i]);
                    } else {
                        right_objects.push(obj);
                        right_index.push(index[i]);
                    }
                }
            }
            Axis::Y => {
                let mid = 0.5 * (bounds.min_y() + bounds.max_y());
                for (i, obj) in (&objects).iter().enumerate() {
                    let above = obj.get_bounds().max_y() - mid;
                    let below = mid - obj.get_bounds().min_y();
                    if above >= below {
                        left_objects.push(obj);
                        left_index.push(index[i]);
                    } else {
                        right_objects.push(obj);
                        right_index.push(index[i]);
                    }
                }
            }
            Axis::Z => {
                let mid = 0.5 * (bounds.min_z() + bounds.max_z());
                for (i, obj) in (&objects).iter().enumerate() {
                    let above = obj.get_bounds().max_z() - mid;
                    let below = mid - obj.get_bounds().min_z();
                    if above >= below {
                        left_objects.push(obj);
                        left_index.push(index[i]);
                    } else {
                        right_objects.push(obj);
                        right_index.push(index[i]);
                    }
                }
            }
        }

        (left_objects, left_index, right_objects, right_index)
    }
}

impl<T: Bounded> Buffer for BVH<T> {
    fn to_buffer(&self) -> Vec<u8> {
        let mut flat: Vec<u8> = Vec::new();

        flat.extend_from_slice(bytemuck::cast_slice(&[self.nodes.len() as u32]));
        flat.extend_from_slice(bytemuck::cast_slice(&[0 as u32; 3]));

        for node in &self.nodes {
            flat.extend_from_slice(bytemuck::cast_slice(&[
                node.bounds.min_x(),
                node.bounds.min_y(),
                node.bounds.min_z(),
            ]));
            flat.extend_from_slice(bytemuck::cast_slice(&[node.node_type as u32]));

            flat.extend_from_slice(bytemuck::cast_slice(&[
                node.bounds.max_x(),
                node.bounds.max_y(),
                node.bounds.max_z(),
            ]));
            flat.extend_from_slice(bytemuck::cast_slice(&[node.ptr]));
        }

        flat
    }

    fn buffer_size(&self) -> usize {
        32 * self.nodes.len() + 16
    }
}

impl Node {
    fn new(bounds: AABB, node_type: NodeType, ptr: u32) -> Self {
        Node {
            bounds,
            node_type,
            ptr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Sphere;
    use glam::Vec3;

    #[test]
    fn test_make_bvh() {
        let s1 = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0, 0);
        let s2 = Sphere::new(Vec3::new(-1.5, 0.0, 0.0), 1.0, 0);
        let s3 = Sphere::new(Vec3::new(1.5, 1.5, 0.0), 1.0, 0);
        let s4 = Sphere::new(Vec3::new(1.5, 3.0, 0.0), 1.0, 0);
        let s5 = Sphere::new(Vec3::new(-1.5, 1.5, 0.0), 1.0, 0);

        let objs = vec![s1, s2, s3, s4, s5];

        let bvh = BVH::from_objects(&objs);

        println!("{:?}", bvh);
    }
}
