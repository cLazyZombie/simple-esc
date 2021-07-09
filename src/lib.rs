#![allow(dead_code)]

use std::cell::RefCell;

#[derive(Debug, PartialEq, Eq)]
struct Health(i32);

struct Name(&'static str);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Entity {
    pub(crate) id: usize,
    pub(crate) gen: u32,
}

struct World {
    entities_count: usize,
    component_vecs: Vec<Box<RefCell<dyn ComponentVec>>>,
    entities: Vec<Option<Entity>>,
    free_entities: Vec<Entity>,
}

impl World {
    fn new() -> Self {
        Self {
            entities_count: 0,
            component_vecs: Vec::new(),
            entities: Vec::new(),
            free_entities: Vec::new(),
        }
    }

    fn new_entity(&mut self) -> Entity {
        if let Some(entity) = self.free_entities.pop() {
            entity
        } else {
            let entity = Entity {
                id: self.entities_count,
                gen: 0,
            };
            self.entities_count += 1;

            for component_vec in self.component_vecs.iter_mut() {
                component_vec.get_mut().push_none();
            }

            entity
        }
    }

    fn add_component_to_entity<ComponentType: 'static>(
        &mut self,
        entity: Entity,
        component: ComponentType,
    ) {
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(components) = (component_vec as &mut dyn std::any::Any)
                .downcast_mut::<Box<RefCell<Components<ComponentType>>>>()
            {
                components.get_mut().set(entity.id, component);
                return;
            }
        }

        // new one
        let mut components = Components::new(self.entities_count);
        components.set(entity.id, component);

        self.component_vecs.push(Box::new(RefCell::new(components)));
    }
}

trait ComponentVec {
    fn push_none(&mut self);
}

struct Components<T: 'static> {
    components: Vec<Option<T>>,
}

impl<T: 'static> Components<T> {
    pub fn new(size: usize) -> Self {
        let mut components = Vec::with_capacity(size);
        for _ in 0..size {
            components.push(None);
        }

        Self { components }
    }

    pub fn set(&mut self, idx: usize, component: T) {
        self.components[idx] = Some(component);
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.components[idx].as_ref()
    }
}

impl<T: 'static> ComponentVec for Components<T> {
    fn push_none(&mut self) {
        self.components.push(None);
    }
}

#[test]
fn test_world() {
    let mut world = World::new();
    let entity = world.new_entity();
    world.add_component_to_entity(entity, Health(10));

    let entity = world.new_entity();
    world.add_component_to_entity(entity, Health(20));

    assert_eq!(world.component_vecs.len(), 1);

    // for (idx, health) in world
    //     .mut_borrow_component_vec::<Health>()
    //     .unwrap()
    //     .iter_mut()
    //     .enumerate()
    // {
    //     if idx == 0 {
    //         let health = health.as_ref().unwrap();
    //         assert_eq!(health, &Health(10));
    //     } else if idx == 1 {
    //         let health = health.as_ref().unwrap();
    //         assert_eq!(health, &Health(20));
    //     }
    // }
}
