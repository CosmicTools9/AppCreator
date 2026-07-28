import * as React from "react";

export const ShadowRootContext = React.createContext<ShadowRoot | null>(null);

export function useShadowRoot(): ShadowRoot | null {
  return React.useContext(ShadowRootContext);
}
