#[cfg(test)]
mod tests {
    use rand::distributions::{Distribution, Uniform};
    use toretsu::task::Task;
    use toretsu::worker::Worker;
    #[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
    struct Job<T> {
        value: T,
        func: fn(T),
    }

    impl<T> Job<T> {
        fn new(value: T, func: fn(T)) -> Self {
            Self { value, func }
        }
    }

    impl<T> Task for Job<T>
    where
        T: Copy,
    {
        fn process(&mut self) {
            (self.func)(self.value)
        }
    }

    fn callback<T: std::fmt::Debug>(item: T) {
        println!("{:?}", item)
    }

    #[test]
    fn simple_example() {
        // Print a list of Rust keywords
        let words: [Job<&str>; 5] =
            ["as", "break", "const", "continue", "crate"].map(|x| Job::new(x, callback));

        let work = Vec::from(words);
        let mut worker = Worker::from(work);
        assert_eq!(worker.queue.len(), 5);

        worker.clock_in();
        assert!(worker.queue.is_empty());

        let more_work = ["else", "extern", "false", "fn", "for"]
            .map(|x| Job::new(x, callback))
            .to_vec();

        worker.assign_many(more_work);

        worker.assign_one(Job::new("if", callback));
        worker.assign_one(Job::new("impl", callback));

        worker.clock_in();
        worker.clock_out();
        assert!(!worker.active);
        assert!(worker.queue.is_empty());
    }

    #[test]
    fn torture_test() {
        let million = 1000 * 1000;
        let ints = {
            let mut ints = Vec::new();
            let range = Uniform::from(0..million);
            let mut rng = rand::thread_rng();

            for _ in 0..million {
                let num = range.sample(&mut rng);
                ints.push(num);
            }

            ints
        };

        let work = ints.into_iter().map(|x| Job::new(x, callback));
        let more_work = work.clone();
        let mut worker = toretsu::worker::Worker::from(work.collect());
        assert_eq!(worker.queue.len(), million);

        worker.clock_in();
        worker.clock_out();
        assert!(!worker.active);
        assert!(worker.queue.is_empty());
    }
}
