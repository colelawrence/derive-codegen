// eslint-disable-next-line @typescript-eslint/no-explicit-any
type $IntentionalAny = any;

/**
 * This special type can help generate pattern matchers for you!
 * Just use it as so:
 *    // enum-ts
 *    type Result<Ok, Err> = Enum<{
 *      Ok: Ok,
 *      Err: Err,
 *    }>
 */
export type Enum<T extends { [Variant: string]: {} }> = {
  [P in keyof T]: Expand<
    Pick<T, P> &
      Omit<
        {
          [K in keyof T]?: never;
        },
        P
      >
  >;
}[keyof T];

type Expand<T> = T extends T
  ? {
      [K in keyof T]: T[K];
    }
  : never;
type U2I<U> = (U extends U ? (u: U) => 0 : never) extends (i: infer I) => 0
  ? Extract<I, U>
  : never;
type ExcludeNevers<T> = {
  [K in keyof T as Required<T>[K] extends never ? never : K]: T[K];
};
type EnumKeysObj<T> = U2I<{
  -readonly [K in keyof T]-?: unknown;
}>;
type EnumKeys<T> = T extends T ? keyof ExcludeNevers<T> : never;
type EnumProp<T, K extends PropertyKey> = Required<
  Pick<Extract<T, Partial<Record<K, unknown>>>, K>
>[K];
type EnumOmit<T, K extends PropertyKey> = Exclude<T, Record<K, unknown>>;
type Match<
  T,
  R = never,
  O = EnumKeysObj<ExcludeNevers<T>>
> = EnumKeys<T> extends never
  ? { $(): R }
  : [R] extends [never]
  ? {
      [P in keyof O]: <R1>(
        cb: (value: EnumProp<T, P>) => R1
      ) => Match<EnumOmit<T, P>, R1>;
    }
  : {
      [P in keyof O]: (
        cb: (value: EnumProp<T, P>) => R
      ) => Match<EnumOmit<T, P>, R>;
    } & {
      _(calculate: (values: T) => R): R;
    };
interface UndefinedMatch<T extends object | undefined | null> {
  nullish<R>(value: () => R): Match<NonNullable<T>, R>;
}
const empty = Symbol("match never");
const THEN = "then";
const EXH = "$";
const OTH = "_";
export function match<T extends object | undefined | null>(
  value: T
): undefined extends T
  ? UndefinedMatch<T>
  : null extends T
  ? UndefinedMatch<T>
  : Match<T> {
  // throw new Error("todo")
  // console.log("variant", { value });
  let found: unknown = empty;
  const proxy = new Proxy(
    {},
    {
      get(_, p): $IntentionalAny {
        // console.log(`get ${String(p)}`);
        if (p === THEN) return undefined; // protect from Promise.resolve(...)
        if (p === EXH) return () => found;
        if (p === OTH)
          return found === empty
            ? (cb: (val: unknown) => unknown) => cb(value)
            : () => found;
        if (found === empty && value && p === (value as any)) {
          return (cb: (inner: unknown) => unknown) => {
            found = cb(null);
            return proxy;
          };
        }
        if (found === empty && value && typeof value === "object" && p in (value as object)) {
          const inner = (value as $IntentionalAny)[p];
          return (cb: (inner: unknown) => unknown) => {
            found = cb(inner);
            return proxy;
          };
        } else {
          return () => proxy;
        }
      },
    }
  );

  if (value == null) {
    return {
      nullish(cb: () => unknown) {
        found = cb();
        return proxy;
      },
    } as $IntentionalAny;
  } else {
    return proxy as $IntentionalAny;
  }
}
