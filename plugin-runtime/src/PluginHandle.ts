import { randomUUID } from 'node:crypto';
import path from 'node:path';
import { Worker } from 'node:worker_threads';
import { PluginInfo } from './plugins';

export interface ParentToWorkerEvent<T = any> {
  callbackId: string;
  name: 'info' | 'run-filter' | 'run-import' | 'run-export' | 'run-http-request-action';
  payload: T;
}

export type WorkerToParentSuccessEvent<T> = {
  callbackId: string;
  payload: T;
};

export type WorkerToParentErrorEvent = {
  callbackId: string;
  error: string;
};

export type WorkerToParentEvent<T = any> = WorkerToParentErrorEvent | WorkerToParentSuccessEvent<T>;

export class PluginHandle {
  readonly pluginDir: string;
  readonly #worker: Worker;

  constructor(pluginDir: string) {
    this.pluginDir = pluginDir;

    const workerPath = path.join(__dirname, 'index.worker.cjs');
    this.#worker = new Worker(workerPath, {
      workerData: {
        pluginDir: this.pluginDir,
      },
    });

    this.#worker.on('error', this.#handleError.bind(this));
    this.#worker.on('exit', this.#handleExit.bind(this));
  }

  async getInfo(): Promise<PluginInfo> {
    return this.invoke('info', null);
  }

  invoke<P, R>(name: ParentToWorkerEvent['name'], payload: P): Promise<R> {
    const callbackId = `cb_${randomUUID().replaceAll('-', '')}`;
    return new Promise((resolve, reject) => {
      const cb = (e: WorkerToParentEvent<R>) => {
        if (e.callbackId !== callbackId) return;

        if ('error' in e) {
          reject(e.error);
        } else {
          resolve(e.payload as R);
        }

        this.#worker.removeListener('message', cb);
      };

      this.#worker.addListener('message', cb);
      this.#worker.postMessage({ callbackId, name, payload });
    });
  }

  async #handleError(err: Error) {
    console.error('Plugin errored', this.pluginDir, err);
  }

  async #handleExit(code: number) {
    if (code === 0) {
      console.log('Plugin exited successfully', this.pluginDir);
    } else {
      console.log('Plugin exited with error', code, this.pluginDir);
    }
  }
}
