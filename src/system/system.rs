pub trait System {
    fn execute(&self, &mut World);
}
