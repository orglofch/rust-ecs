use std::collections::HashMap;

pub struct World {
    components: Vec<u32>,

    entity_offset: usize,
}

impl World {
    /*pub fn new() -> World {
        World {}
    }*/

    pub fn create_entity(&mut self) {}

    //pub fn register<T: Component>(&mut self) {}

    //pub fn unregister<T: Component>(&mut self) {}
}

struct SystemManager {}

impl SystemManager {
    //fn register_service(&self, service: Service) {}
}
