use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct VoxelHashMap {
    pub map: HashMap<(i64, i64), HashSet<Entity>>,
    pub cell_size: f32,
}

impl VoxelHashMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            cell_size: 1.,
        }
    }

    pub fn vec2_to_key(&self, vec: Vec2) -> (i64, i64) {
        (
            (vec.x / self.cell_size).floor() as i64,
            (vec.y / self.cell_size).floor() as i64,
        )
    }

    pub fn key_to_vec2(&self, key: (i64, i64)) -> Vec2 {
        Vec2::new(key.0 as f32 * self.cell_size, key.1 as f32 * self.cell_size)
    }

    pub fn get_neighbor_keys(&self, vec: Vec2) -> Vec<(i64, i64)> {
        let key = self.vec2_to_key(vec);
        let mut neighbours = Vec::new();
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue;
                }
                let neighbour_key = (key.0 + i, key.1 + j);
                neighbours.push(neighbour_key);
            }
        }
        neighbours
    }

    pub fn get_neighbor_keys_within(&self, vec: Vec2, radius: f32) -> Vec<(i64, i64)> {
        let key = self.vec2_to_key(vec);
        let radius = (radius / self.cell_size).ceil() as i64;
        // let mut neighbours = Vec::new();
        // for i in -radius..=radius {
        //     for j in -radius..=radius {
        //         let neighbour_key = (key.0 + i, key.1 + j);
        //         neighbours.push(neighbour_key);
        //     }
        // }
        // neighbours
        (-radius..=radius)
            .flat_map(|i| (-radius..=radius).map(move |j| (key.0 + i, key.1 + j)))
            .collect()
    }

    pub fn get_neighbor_entities(&self, vec: Vec2) -> Vec<Entity> {
        // let mut entities = HashSet::new();
        // let neighbours = self.get_neighbor_keys(vec);
        // for neighbour in neighbours {
        //     if let Some(neighbour_entities) = self.map.get(&neighbour) {
        //         entities.extend(neighbour_entities.iter());
        //     }
        // }
        // entities

        self.get_neighbor_keys(vec)
            .iter()
            .filter_map(|key| self.map.get(key))
            .flat_map(|entities| entities.iter())
            .copied()
            .collect()
    }

    pub fn insert(&mut self, vec: Vec2, entity: Entity) {
        let key = self.vec2_to_key(vec);
        self.map.entry(key).or_default().insert(entity);
    }

    pub fn contains(&self, vec: Vec2, entity: Entity) -> bool {
        let key = self.vec2_to_key(vec);
        if let Some(entities) = self.map.get(&key) {
            entities.contains(&entity)
        } else {
            false
        }
    }

    pub fn remove(&mut self, vec: Vec2, entity: Entity) {
        let key = self.vec2_to_key(vec);
        if let Some(entities) = self.map.get_mut(&key) {
            entities.remove(&entity);

            if entities.is_empty() {
                self.map.remove(&key);
            }
        }
    }

    pub fn move_to(&mut self, old_vec: Vec2, new_vec: Vec2, entity: Entity) {
        self.remove(old_vec, entity);
        self.insert(new_vec, entity);
    }

    pub fn update_entity(&mut self, old_vec: Vec2, new_vec: Vec2, entity: Entity) {
        let old_key = self.vec2_to_key(old_vec);
        let new_key = self.vec2_to_key(new_vec);
        if old_key == new_key {
            return;
        }

        self.move_to(old_vec, new_vec, entity);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vec2_to_key() {
        // Test the conversion of a Vec2 to a key
        let voxel = VoxelHashMap::new();
        assert_eq!(voxel.vec2_to_key(Vec2::new(0.0, 0.0)), (0, 0));
        assert_eq!(voxel.vec2_to_key(Vec2::new(1.8, 1.0)), (1, 1));
        assert_eq!(voxel.vec2_to_key(Vec2::new(1.0, 0.0)), (1, 0));
        assert_eq!(voxel.vec2_to_key(Vec2::new(0.0, 1.1)), (0, 1));
        assert_eq!(voxel.vec2_to_key(Vec2::new(0.0, 11.)), (0, 11));
        assert_eq!(voxel.vec2_to_key(Vec2::new(18., 1.0)), (18, 1));
    }

    #[test]
    fn test_vec2_to_key_with_voxel_dim() {
        // Test the conversion of a Vec2 to a key with a voxel dimension
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 2.;
        assert_eq!(voxel.vec2_to_key(Vec2::new(0.0, 0.0)), (0, 0));
        assert_eq!(voxel.vec2_to_key(Vec2::new(1.8, 1.0)), (0, 0));
        assert_eq!(voxel.vec2_to_key(Vec2::new(1.0, 0.0)), (0, 0));
        assert_eq!(voxel.vec2_to_key(Vec2::new(0.0, 1.1)), (0, 0));
        assert_eq!(voxel.vec2_to_key(Vec2::new(0.0, 11.)), (0, 5));
        assert_eq!(voxel.vec2_to_key(Vec2::new(18., 1.0)), (9, 0));
    }

    #[test]
    fn test_key_to_vec2() {
        // Test the conversion of a key to a Vec2
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 12.5;
        assert_eq!(voxel.key_to_vec2((0, 0)), Vec2::new(0.0, 0.0));
        assert_eq!(voxel.key_to_vec2((1, 1)), Vec2::new(12.5, 12.5));
        assert_eq!(voxel.key_to_vec2((1, 0)), Vec2::new(12.5, 0.0));
        assert_eq!(voxel.key_to_vec2((0, 1)), Vec2::new(0.0, 12.5));
        assert_eq!(voxel.key_to_vec2((0, 11)), Vec2::new(0.0, 137.5));
        assert_eq!(voxel.key_to_vec2((18, 1)), Vec2::new(225.0, 12.5));
    }

    #[test]
    fn test_get_neighbor_keys() {
        // Test the retrieval of the keys of the neighbours of a voxel
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 10.;

        let neighbours = voxel.get_neighbor_keys(Vec2::new(0.0, 0.0));
        assert_eq!(neighbours.len(), 8);
        assert!(neighbours.contains(&(1, 0)));
        assert!(neighbours.contains(&(1, 1)));
        assert!(neighbours.contains(&(0, 1)));
        assert!(neighbours.contains(&(-1, 1)));
        assert!(neighbours.contains(&(-1, 0)));
        assert!(neighbours.contains(&(-1, -1)));
        assert!(neighbours.contains(&(0, -1)));
        assert!(neighbours.contains(&(1, -1)));

        let neighbours = voxel.get_neighbor_keys(Vec2::new(50.0, 50.0));
        assert_eq!(neighbours.len(), 8);
        assert!(neighbours.contains(&(6, 5)));
        assert!(neighbours.contains(&(6, 6)));
        assert!(neighbours.contains(&(5, 6)));
        assert!(neighbours.contains(&(4, 6)));
        assert!(neighbours.contains(&(4, 5)));
        assert!(neighbours.contains(&(4, 4)));
        assert!(neighbours.contains(&(5, 4)));
        assert!(neighbours.contains(&(6, 4)));
    }

    #[test]
    fn test_get_neighbor_entities() {
        // Test the retrieval of the entities in the neighbours of a voxel
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 10.;

        let entity_1 = Entity::from_raw(0);
        voxel.insert(Vec2::new(55.0, 20.0), entity_1);

        let entity_2 = Entity::from_raw(1);
        voxel.insert(Vec2::new(60.0, 30.0), entity_2);

        let entity_3 = Entity::from_raw(2);
        voxel.insert(Vec2::new(65.0, 40.0), entity_3);

        let entities = voxel.get_neighbor_entities(Vec2::new(55.0, 20.0));
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&entity_2));
        assert!(!entities.contains(&entity_1));
        assert!(!entities.contains(&entity_3));

        let entities = voxel.get_neighbor_entities(Vec2::new(60.0, 30.0));
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&entity_1));
        assert!(entities.contains(&entity_3));
        assert!(!entities.contains(&entity_2));

        let entities = voxel.get_neighbor_entities(Vec2::new(65.0, 40.0));
        assert_eq!(entities.len(), 1);
        assert!(entities.contains(&entity_2));
        assert!(!entities.contains(&entity_1));
        assert!(!entities.contains(&entity_3));
    }

    #[test]
    fn test_insert() {
        // Test the insertion of an entity in the voxel map
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 10.;

        let entity_1 = Entity::from_raw(0);
        voxel.insert(Vec2::new(55.0, 20.0), entity_1);

        assert_eq!(voxel.map.len(), 1);
        assert_eq!(voxel.map.get(&(5, 2)).unwrap().len(), 1);
        assert!(voxel.map.get(&(5, 2)).unwrap().contains(&entity_1));

        let entity_2 = Entity::from_raw(1);
        voxel.insert(Vec2::new(59.9, 29.9), entity_2);

        assert_eq!(voxel.map.len(), 1); // The entity is in the same voxel
        assert_eq!(voxel.map.get(&(5, 2)).unwrap().len(), 2);
        assert!(voxel.map.get(&(5, 2)).unwrap().contains(&entity_1));
        assert!(voxel.map.get(&(5, 2)).unwrap().contains(&entity_2));

        let entity_3 = Entity::from_raw(2);
        voxel.insert(Vec2::new(60.0, 30.0), entity_3);

        assert_eq!(voxel.map.len(), 2); // The entity is in another voxel
        assert_eq!(voxel.map.get(&(5, 2)).unwrap().len(), 2);
        assert!(voxel.map.get(&(5, 2)).unwrap().contains(&entity_1));
        assert!(voxel.map.get(&(5, 2)).unwrap().contains(&entity_2));
        assert_eq!(voxel.map.get(&(6, 3)).unwrap().len(), 1);
        assert!(voxel.map.get(&(6, 3)).unwrap().contains(&entity_3));
        assert!(!voxel.map.get(&(5, 2)).unwrap().contains(&entity_3));
    }

    #[test]
    fn test_contains() {
        // Test the check if an entity is in a voxel
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 10.;

        let entity_1 = Entity::from_raw(0);
        voxel.insert(Vec2::new(55.0, 20.0), entity_1);

        assert!(voxel.contains(Vec2::new(55.0, 20.0), entity_1));
        assert!(!voxel.contains(Vec2::new(55.0, 20.0), Entity::from_raw(1)));
    }

    #[test]
    fn test_remove() {
        // Test the removal of an entity from a voxel
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 10.;

        let entity_1 = Entity::from_raw(0);
        voxel.insert(Vec2::new(55.0, 20.0), entity_1);

        assert_eq!(voxel.map.len(), 1);
        assert_eq!(voxel.map.get(&(5, 2)).unwrap().len(), 1);
        assert!(voxel.map.get(&(5, 2)).unwrap().contains(&entity_1));

        voxel.remove(Vec2::new(55.0, 20.0), entity_1);

        assert_eq!(voxel.map.len(), 0);
        let result = std::panic::catch_unwind(|| voxel.map.get(&(5, 2)).unwrap().len());
        assert!(result.is_err());
    }

    #[test]
    fn test_move_to() {
        // Test the movement of an entity from a voxel to another
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 10.;

        let entity_1 = Entity::from_raw(0);
        voxel.insert(Vec2::new(55.0, 20.0), entity_1);

        assert_eq!(voxel.map.len(), 1);
        assert_eq!(voxel.map.get(&(5, 2)).unwrap().len(), 1);
        assert!(voxel.map.get(&(5, 2)).unwrap().contains(&entity_1));

        voxel.move_to(Vec2::new(55.0, 20.0), Vec2::new(60.0, 30.0), entity_1);

        assert_eq!(voxel.map.len(), 1);
        assert!(voxel.map.get(&(5, 2)).is_none());
        assert_eq!(voxel.map.get(&(6, 3)).unwrap().len(), 1);
        assert!(voxel.map.get(&(6, 3)).unwrap().contains(&entity_1));
    }

    #[test]
    fn test_update_entity() {
        // Test the update of the position of an entity in the voxel map
        let mut voxel = VoxelHashMap::new();
        voxel.cell_size = 10.;

        let entity_1 = Entity::from_raw(0);
        voxel.insert(Vec2::new(55.0, 20.0), entity_1);

        assert_eq!(voxel.map.len(), 1);
        assert_eq!(voxel.map.get(&(5, 2)).unwrap().len(), 1);
        assert!(voxel.map.get(&(5, 2)).unwrap().contains(&entity_1));

        voxel.update_entity(Vec2::new(55.0, 20.0), Vec2::new(60.0, 30.0), entity_1);

        assert_eq!(voxel.map.len(), 1);
        assert!(voxel.map.get(&(5, 2)).is_none());
        assert_eq!(voxel.map.get(&(6, 3)).unwrap().len(), 1);
        assert!(voxel.map.get(&(6, 3)).unwrap().contains(&entity_1));

        voxel.update_entity(Vec2::new(60.0, 30.0), Vec2::new(65.0, 40.0), entity_1);

        assert_eq!(voxel.map.len(), 1);
        assert!(voxel.map.get(&(6, 3)).is_none());
        assert_eq!(voxel.map.get(&(6, 4)).unwrap().len(), 1);
        assert!(voxel.map.get(&(6, 4)).unwrap().contains(&entity_1));
    }
}
