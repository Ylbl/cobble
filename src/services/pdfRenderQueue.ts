type RenderTask = () => Promise<void>;

class PdfRenderQueue {
  private queue: RenderTask[] = [];
  private running = 0;
  private maxConcurrent: number;

  constructor(maxConcurrent = 2) {
    this.maxConcurrent = maxConcurrent;
  }

  enqueue(task: RenderTask) {
    this.queue.push(task);
    this.flush();
  }

  clear() {
    this.queue = [];
  }

  private flush() {
    while (this.running < this.maxConcurrent && this.queue.length > 0) {
      const task = this.queue.shift();
      if (!task) break;
      this.running++;
      task()
        .catch(() => {})
        .finally(() => {
          this.running--;
          this.flush();
        });
    }
  }
}

// Global singleton
export const pdfRenderQueue = new PdfRenderQueue(2);
