pub struct CircularQueue<T: Copy, const SIZE: usize> {
    buffer: [Option<T>; SIZE],
    capacity: usize,
    front: usize,
    rear: usize,
}

impl<T: Copy, const SIZE: usize> CircularQueue<T, SIZE> {
    pub const fn new() -> Self {
        CircularQueue {
            buffer: [None; SIZE],
            capacity: SIZE - 1,
            front: 0,
            rear: 0,
        }
    }

    /// 压入队列,没有返回值,不允许失败,会循环覆盖先压入的元素
    pub fn enqueue(&mut self, item: T) {
        // 直接丢弃先压入的字节
        while self.is_full() {
            let _ = self.dequeue();
        }

        let next_rear = (self.rear + 1) % SIZE;
        self.buffer[self.rear] = Some(item);
        self.rear = next_rear;
    }

    pub fn dequeue(&mut self) -> Result<Option<T>, &'static str> {
        if self.front == self.rear && self.buffer[self.front].is_none() {
            return Err("Queue is empty");
        }

        let item = self.buffer[self.front].take();
        self.front = (self.front + 1) % SIZE;
        Ok(item)
    }

    pub fn is_empty(&self) -> bool {
        self.front == self.rear && self.buffer[self.front].is_none()
    }

    pub fn is_full(&self) -> bool {
        let next_rear = (self.rear + 1) % SIZE;
        next_rear == self.front
    }
}
