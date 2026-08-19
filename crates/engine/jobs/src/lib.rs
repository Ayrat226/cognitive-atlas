//! Job system with work-stealing thread pool

use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;
use crossbeam::deque::{Injector, Worker};
use tracing::{span, Level};

/// Task priority levels
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Priority {
    Background = 0,
    AsyncIO = 1,
    RenderPrep = 2,
    SimFixed = 3,
}

/// Task trait for execution
pub trait Task: Send + Sync {
    fn execute(&self);
    fn priority(&self) -> Priority { Priority::Background }
    fn name(&self) -> &'static str { "unnamed" }
}

impl<F> Task for F
where
    F: Fn() + Send + Sync,
{
    fn execute(&self) {
        self();
    }
}

/// Boxed task for type erasure
pub type BoxedTask = Arc<dyn Task + Send + Sync>;

/// Work-stealing thread pool
pub struct JobPool {
    injector: Arc<Injector<BoxedTask>>,
    handles: Vec<thread::JoinHandle<()>>,
    shutdown: Arc<Mutex<bool>>,
    shutdown_cond: Arc<Condvar>,
    num_threads: usize,
}

impl JobPool {
    /// Create new job pool with given number of threads
    pub fn new(num_threads: usize) -> Self {
        let injector = Arc::new(Injector::new());
        let mut handles = Vec::new();
        let shutdown = Arc::new(Mutex::new(false));
        let shutdown_cond = Arc::new(Condvar::new());

        for i in 0..num_threads {
            let worker = Worker::new_fifo();
            let injector = injector.clone();
            let shutdown = shutdown.clone();
            let shutdown_cond = shutdown_cond.clone();

            let handle = thread::Builder::new()
                .name(format!("lithos-job-{}", i))
                .spawn(move || {
                    Self::worker_loop(worker, injector, shutdown, shutdown_cond);
                })
                .unwrap();

            handles.push(handle);
        }

        Self {
            injector,
            handles,
            shutdown,
            shutdown_cond,
            num_threads,
        }
    }

    fn worker_loop(
        worker: Worker<BoxedTask>,
        injector: Arc<Injector<BoxedTask>>,
        shutdown: Arc<Mutex<bool>>,
        shutdown_cond: Arc<Condvar>,
    ) {
        loop {
            let task = worker.pop()
                .or_else(|| injector.steal().success())
                .or_else(|| injector.steal().success());

            if let Some(task) = task {
                let _span = span!(Level::TRACE, "job", name = task.name());
                let _enter = _span.enter();
                task.execute();
            } else {
                let mut guard = shutdown.lock().unwrap();
                if *guard {
                    break;
                }
                let _ = shutdown_cond.wait_timeout(guard, Duration::from_millis(1)).unwrap();
            }
        }
    }

    pub fn submit<T: Task + 'static>(&self, task: T) {
        self.injector.push(Arc::new(task));
    }

    pub fn num_threads(&self) -> usize {
        self.num_threads
    }

    pub fn pending_tasks(&self) -> usize {
        self.injector.len()
    }

    pub fn wait_idle(&self) {
        loop {
            if self.injector.is_empty() {
                return;
            }
            thread::yield_now();
        }
    }

    pub fn shutdown(mut self) {
        {
            let mut guard = self.shutdown.lock().unwrap();
            *guard = true;
        }
        self.shutdown_cond.notify_all();
        for handle in self.handles.drain(..) {
            handle.join().unwrap();
        }
    }
}

impl Drop for JobPool {
    fn drop(&mut self) {
        let mut guard = self.shutdown.lock().unwrap();
        *guard = true;
        self.shutdown_cond.notify_all();
        for handle in self.handles.drain(..) {
            handle.join().unwrap();
        }
    }
}

/// Task graph for dependent tasks
pub struct TaskGraph {
    nodes: Vec<TaskNode>,
    edges: Vec<(usize, usize)>,
    indegree: Vec<usize>,
}

struct TaskNode {
    task: BoxedTask,
    priority: Priority,
}

impl TaskGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new(), indegree: Vec::new() }
    }

    pub fn add_task<T: Task + 'static>(&mut self, task: T) -> usize {
        let idx = self.nodes.len();
        let priority = task.priority();
        self.nodes.push(TaskNode { task: Arc::new(task), priority });
        self.indegree.push(0);
        idx
    }

    pub fn add_dependency(&mut self, from: usize, to: usize) {
        self.edges.push((from, to));
        self.indegree[to] += 1;
    }

    pub fn execute(self, pool: &JobPool) {
        let node_count = self.nodes.len();
        let mut ready = Vec::new();
        let mut indegree = self.indegree.clone();
        let edges = self.edges;
        let nodes = self.nodes;

        for (i, &deg) in indegree.iter().enumerate() {
            if deg == 0 {
                ready.push(i);
            }
        }

        ready.sort_by_key(|&i| std::cmp::Reverse(nodes[i].priority));

        let remaining = Arc::new(Mutex::new(ready.len()));
        let cond = Arc::new(Condvar::new());
        let pool_injector = pool.injector.clone();
        let edges = edges;
        let indegree = Arc::new(Mutex::new(indegree));
        let completed = Arc::new(Mutex::new(vec![false; node_count]));
        let nodes_arc = Arc::new(nodes);

        for &node_idx in &ready {
            let task = nodes_arc[node_idx].task.clone();
            let pool_injector = pool.injector.clone();
            let remaining = Arc::new(Mutex::new(1));
            let cond = Arc::new(Condvar::new());
            let edges = edges.clone();
            let indegree = Arc::new(Mutex::new(vec![0; node_count])); // Simplified
            let completed = Arc::new(Mutex::new(vec![false; node_count]));
            let nodes_arc = nodes_arc.clone();

            pool_injector.push(Arc::new(move || {
                task.execute();
                let mut deg = indegree.lock().unwrap();
                let mut comp = completed.lock().unwrap();
                comp[node_idx] = true;
                for &(from, to) in edges.iter() {
                    if from == node_idx {
                        deg[to] -= 1;
                        if deg[to] == 0 {
                            // Simplified - real implementation would submit dependent task
                        }
                    }
                }
                let mut rem = remaining.lock().unwrap();
                *rem -= 1;
                if *rem == 0 {
                    cond.notify_all();
                }
            }));
        }
    }
}

/// Simple parallel for loop
pub fn parallel_for<F>(pool: &JobPool, range: std::ops::Range<usize>, chunk_size: usize, f: F)
where
    F: Fn(usize) + Send + Sync + 'static,
{
    let f = Arc::new(f);
    let total = range.len();
    let chunks = (total + chunk_size - 1) / chunk_size;

    for chunk in 0..chunks {
        let start = range.start + chunk * chunk_size;
        let end = (start + chunk_size).min(range.end);
        let f = f.clone();
        pool.injector.push(Arc::new(move || {
            for i in start..end {
                f(i);
            }
        }));
    }
}

static mut GLOBAL_POOL: Option<JobPool> = None;

pub fn init_global_pool(num_threads: usize) {
    unsafe {
        GLOBAL_POOL = Some(JobPool::new(num_threads));
    }
}

pub fn global_pool() -> &'static JobPool {
    unsafe {
        GLOBAL_POOL.as_ref().expect("Global job pool not initialized")
    }
}

pub fn submit<T: Task + 'static>(task: T) {
    global_pool().submit(task);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_job_pool_basic() {
        let pool = JobPool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..100 {
            let c = counter.clone();
            pool.submit(move || {
                c.fetch_add(1, Ordering::Relaxed);
            });
        }

        pool.wait_idle();
        assert_eq!(counter.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_parallel_for() {
        let pool = JobPool::new(4);
        let data = Arc::new(Mutex::new(vec![0; 1000]));

        parallel_for(&pool, 0..1000, 32, {
            let d = data.clone();
            move |i| {
                d.lock().unwrap()[i] = i * 2;
            }
        });

        pool.wait_idle();
        for i in 0..1000 {
            assert_eq!(data.lock().unwrap()[i], i * 2);
        }
    }
}