export type Resolvable<T = unknown> = Promise<T> & {
  resolve: (
    // eslint-disable-next-line no-unused-vars
    value: T
  ) => void;
};

export function resolvable<T>(): Resolvable<T> {
  let resolve;
  const promise: any = new Promise((r) => (resolve = r));
  promise.resolve = resolve;
  return promise;
}
