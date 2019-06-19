use std::sync::{mpsc, Arc, Mutex};

trait MappingBox<In, Out> {
    fn map(self: Box<Self>, input: In) -> Out;
}

impl<In, Out, F: Fn(In) -> Out> MappingBox<In, Out> for F {
    fn map(self: Box<F>, input: In) -> Out {
        (*self)(input)
    }
}

type Mapper<In, Out> = Box<dyn MappingBox<In, Out> + Send + 'static>;

struct Worker<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    input_queue: Arc<Mutex<mpsc::Receiver<In>>>,
    output_queue: Arc<Mutex<mpsc::Sender<Out>>>,
    mapper: Mapper<In, Out>,
}

impl<In, Out> Worker<In, Out>
where
    In: Send + 'static,
    Out: Send + 'static,
{
    pub fn new(
        input_queue: Arc<Mutex<mpsc::Receiver<In>>>,
        output_queue: Arc<Mutex<mpsc::Sender<Out>>>,
        mapper: Mapper<In, Out>,
    ) -> Self {
        Worker {
            input_queue,
            output_queue,
            mapper,
        }
    }
}
