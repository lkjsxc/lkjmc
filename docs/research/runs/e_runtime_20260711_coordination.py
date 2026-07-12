import queue
import threading


class Direct:
    def call(self, _instance, work):
        return work()

    def close(self):
        pass


class Keyed:
    def __init__(self):
        self.locks, self.guard = {}, threading.Lock()

    def call(self, instance, work):
        with self.guard:
            lock = self.locks.setdefault(instance, threading.Lock())
        with lock:
            return work()

    def close(self):
        pass


class Mailbox:
    def __init__(self):
        self.queues, self.workers, self.lock = {}, {}, threading.Lock()

    def call(self, instance, work):
        with self.lock:
            if instance not in self.queues:
                tasks = queue.Queue()
                worker = threading.Thread(target=self.serve, args=(tasks,), daemon=True)
                self.queues[instance], self.workers[instance] = tasks, worker
                worker.start()
            tasks = self.queues[instance]
        done, result = threading.Event(), []
        tasks.put((work, done, result))
        if not done.wait(timeout=3):
            raise TimeoutError(f"mailbox timed out for {instance}")
        return result[0]

    @staticmethod
    def serve(tasks):
        while True:
            task = tasks.get()
            if task is None:
                return
            work, done, result = task
            result.append(work())
            done.set()

    def close(self):
        for tasks in self.queues.values():
            tasks.put(None)
        for worker in self.workers.values():
            worker.join(timeout=1)
