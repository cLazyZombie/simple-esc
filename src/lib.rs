#![allow(dead_code)]

use std::{cell::RefCell, fmt::Debug, ops::{Deref, DerefMut}};

#[derive(Debug, PartialEq, Eq)]
struct Health(i32);

#[derive(Debug, PartialEq, Eq)]
struct Speed(i32);

struct Name(&'static str);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Entity {
    pub(crate) id: usize,
    pub(crate) gen: u32,
}

struct World {
    entities_count: usize,
    component_vecs: Vec<Box<dyn ComponentVec>>,
    entities: Vec<Entity>,
    free_entities: Vec<Entity>,
}

fn print_type<T>(_: T) {
    println!("type: {}", std::any::type_name::<T>());
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
            self.entities[entity.id] = entity;
            entity
        } else {
            let entity = Entity {
                id: self.entities_count,
                gen: 0,
            };
            self.entities_count += 1;

            for component_vec in self.component_vecs.iter_mut() {
                component_vec.push_none();
            }

            self.entities.push(entity);
            entity
        }
    }

    fn add_component_to_entity<ComponentType: 'static + std::fmt::Debug>(
        &mut self,
        entity: Entity,
        component: ComponentType,
    ) {
        for component_vec in self.component_vecs.iter_mut() {
            if let Some(components) = component_vec.as_any_mut().downcast_mut::<Components<ComponentType>>() {
                components.set(entity.id, component);
                return;
            }
        }

        // new one
        let mut components = Components::new(self.entities_count);
        components.set(entity.id, component);

        self.component_vecs.push(Box::new(components));
    }

    fn get_component<ComponentType: 'static>(&self, entity: Entity) -> Option<RefComponent<ComponentType>>{
        if let Some(ent) = self.entities.get(entity.id) {
            if *ent == entity {
                for components in self.component_vecs.iter() {
                    if let Some(components) = components.as_any().downcast_ref::<Components<ComponentType>>() {
                        return Some(components.get(entity.id));
                    }
                }
            }
        }

        None
    }

    fn get_component_mut<ComponentType: 'static>(&self, entity: Entity) -> Option<RefMutComponent<ComponentType>> {
        if let Some(ent) = self.entities.get(entity.id) {
            if *ent == entity{
                for components in self.component_vecs.iter() {
                    if let Some(components) = components.as_any().downcast_ref::<Components<ComponentType>>() {
                        return Some(components.get_mut(entity.id));
                    }
                }
            }
        }

        None
    }
}

trait ComponentVec : Debug{
    fn push_none(&mut self);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

#[derive(Debug)]
struct Components<T: 'static> {
    components: RefCell<Vec<Option<T>>>,
}

impl<T: 'static> Components<T> {
    pub fn new(size: usize) -> Self {
        let mut components = RefCell::new(Vec::with_capacity(size));
        for _ in 0..size {
            components.get_mut().push(None);
        }

        Self { components }
    }

    pub fn set(&mut self, idx: usize, component: T) {
        self.components.get_mut()[idx] = Some(component);
    }

    pub fn get(&self, idx: usize) -> RefComponent<T> {
        let r = self.components.borrow();
        RefComponent {
            refer: r,
            idx: idx,
        }
    }

    pub fn get_mut(&self, idx: usize) -> RefMutComponent<T> {
        let r = self.components.borrow_mut();
        RefMutComponent {
            r,
            idx,
        }
    }
}

impl<T: 'static + std::fmt::Debug + std::any::Any> ComponentVec for Components<T> {
    fn push_none(&mut self) {
        self.components.get_mut().push(None);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
}

struct RefComponent<'a, T: 'static> {
    refer: std::cell::Ref<'a, Vec<Option<T>>>,
    idx: usize,
}

impl<'a, T: 'static> std::ops::Deref for RefComponent<'a, T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        self.refer.get(self.idx).unwrap()
    }
}

struct RefMutComponent<'a, T: 'static> {
    r: std::cell::RefMut<'a, Vec<Option<T>>>,
    idx: usize,
}

impl <'a, T: 'static> Deref for RefMutComponent<'a, T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        self.r.get(self.idx).unwrap()
    }
}

impl <'a, T: 'static> DerefMut for RefMutComponent<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.r.get_mut(self.idx).unwrap()
    }
}

#[test]
fn test_world() {
    let mut world = World::new();
    let entity_1 = world.new_entity();
    world.add_component_to_entity(entity_1, Health(10));

    let entity_2 = world.new_entity();
    world.add_component_to_entity(entity_2, Health(20));
    world.add_component_to_entity(entity_2, Speed(100));

    assert_eq!(world.component_vecs.len(), 2);

    let health_comp_1 = world.get_component::<Health>(entity_1).unwrap();
    let comp1 = health_comp_1.as_ref();
    assert_eq!(comp1.unwrap().0, 10);

    let health_comp_2 = world.get_component::<Health>(entity_2).unwrap();
    let comp2 = health_comp_2.deref().as_ref().unwrap();
    assert_eq!(comp2.0, 20);

    let mut speed_comp_2 = world.get_component_mut::<Speed>(entity_2).unwrap();
    let speed = speed_comp_2.deref_mut().as_mut().unwrap();
    speed.0 = 1000;
}

#[test]
fn test_downcast() {

    #[derive(Debug)]
    struct MyS(i32);

    // let a = MyS(10);
    // let b: &dyn std::any::Any = &a;
    // let c = b.downcast_ref::<MyS>().unwrap();

    let a : Box<dyn std::any::Any> = Box::new(MyS(10));
    let b: &dyn std::any::Any = Box::as_ref(&a) as &dyn std::any::Any;
    let b = b.downcast_ref::<MyS>();
    let _ = b.unwrap();
    
    // let a: Box<RefCell<dyn Debug>> = Box::new(RefCell::new(MyS(10)));
    // let b = (&a as &dyn std::any::Any).downcast_ref::<Box<RefCell<MyS>>>();
    // let b = b.unwrap();
}