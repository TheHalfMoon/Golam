declare module 'react' {
  export interface ReactElement {}
  export type ReactNode = ReactElement | string | number | boolean | null | undefined | ReactNode[];
  export type StateSetter<T> = (value: T | ((previous: T) => T)) => void;

  export function createElement(
    type: string | ((props: Record<string, unknown>) => ReactElement),
    props?: Record<string, unknown> | null,
    ...children: ReactNode[]
  ): ReactElement;

  export function useEffect(effect: () => void | (() => void), dependencies?: readonly unknown[]): void;
  export function useState<T>(initial: T): [T, StateSetter<T>];
}

declare module 'react-dom/client' {
  import type { ReactElement } from 'react';

  export interface Root {
    render(element: ReactElement): void;
  }

  export function createRoot(container: Element | DocumentFragment): Root;
}
