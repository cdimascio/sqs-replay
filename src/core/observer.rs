pub trait IVisitor<T> {
    fn visit(&self, o: &T);
}

pub trait ISubject<'a, O, T: IVisitor<O>> {
    fn attach(&mut self, observer: &'a T);
    fn detach(&mut self, observer: &'a T);
    fn notify_observers(&self, o: &O);
}